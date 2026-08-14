use monitoring::{MonitorSnapshot, PerformanceBaseline, PerformanceDeviation};
use serde::{Deserialize, Serialize};

const PERCENT_ELEVATED_DELTA: f64 = 5.0;
const PERCENT_SIGNIFICANT_DELTA: f64 = 15.0;
const RELATIVE_ELEVATED: f64 = 0.25;
const RELATIVE_SIGNIFICANT: f64 = 0.50;
const IO_ELEVATED_BYTES_PER_SECOND: f64 = 1_048_576.0;
const IO_SIGNIFICANT_BYTES_PER_SECOND: f64 = 10_485_760.0;
const PROCESS_ELEVATED_COUNT: usize = 10;
const PROCESS_SIGNIFICANT_COUNT: usize = 50;
const RUNNING_ELEVATED_COUNT: usize = 2;
const RUNNING_SIGNIFICANT_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceAnomalyLevel {
    Normal,
    Elevated,
    Significant,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PerformanceMetric {
    Cpu,
    Memory,
    StorageRead,
    StorageWrite,
    ProcessCount,
    RunningProcesses,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceAnomaly {
    pub metric: PerformanceMetric,
    pub level: PerformanceAnomalyLevel,
    pub current_value: f64,
    pub baseline_value: f64,
    pub deviation: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceAnomalyReport {
    pub overall: PerformanceAnomalyLevel,
    pub anomalies: Vec<PerformanceAnomaly>,
    pub summary: String,
}

pub fn explain(snapshot: &MonitorSnapshot) -> PerformanceAnomalyReport {
    let Some(baseline) = snapshot.baseline.as_ref() else {
        return PerformanceAnomalyReport {
            overall: PerformanceAnomalyLevel::Normal,
            anomalies: Vec::new(),
            summary: "Collecting enough local history to establish a performance baseline.".to_owned(),
        };
    };

    let deviation = snapshot.deviation.as_ref().unwrap_or(&PerformanceDeviation {
        cpu_percent: snapshot.cpu_percent - baseline.cpu_percent,
        memory_percent: snapshot.memory_percent - baseline.memory_percent,
        storage_read_bytes_per_second:
            snapshot.storage_read_bytes_per_second - baseline.storage_read_bytes_per_second,
        storage_write_bytes_per_second:
            snapshot.storage_write_bytes_per_second - baseline.storage_write_bytes_per_second,
        process_count: snapshot.process_count.abs_diff(baseline.process_count),
        running_processes: snapshot.running_processes.abs_diff(baseline.running_processes),
    });

    let mut anomalies = vec![
        percent_anomaly(
            PerformanceMetric::Cpu,
            snapshot.cpu_percent,
            baseline.cpu_percent,
            deviation.cpu_percent,
            "CPU utilization",
        ),
        percent_anomaly(
            PerformanceMetric::Memory,
            snapshot.memory_percent,
            baseline.memory_percent,
            deviation.memory_percent,
            "Memory utilization",
        ),
        io_anomaly(
            PerformanceMetric::StorageRead,
            snapshot.storage_read_bytes_per_second,
            baseline.storage_read_bytes_per_second,
            deviation.storage_read_bytes_per_second,
            "Storage read throughput",
        ),
        io_anomaly(
            PerformanceMetric::StorageWrite,
            snapshot.storage_write_bytes_per_second,
            baseline.storage_write_bytes_per_second,
            deviation.storage_write_bytes_per_second,
            "Storage write throughput",
        ),
        count_anomaly(
            PerformanceMetric::ProcessCount,
            snapshot.process_count,
            baseline.process_count,
            deviation.process_count,
            "Process count",
            PROCESS_ELEVATED_COUNT,
            PROCESS_SIGNIFICANT_COUNT,
        ),
        count_anomaly(
            PerformanceMetric::RunningProcesses,
            snapshot.running_processes,
            baseline.running_processes,
            deviation.running_processes,
            "Running process count",
            RUNNING_ELEVATED_COUNT,
            RUNNING_SIGNIFICANT_COUNT,
        ),
    ];

    let overall = anomalies
        .iter()
        .map(|anomaly| anomaly.level)
        .max()
        .unwrap_or(PerformanceAnomalyLevel::Normal);

    let significant = anomalies
        .iter()
        .filter(|anomaly| anomaly.level == PerformanceAnomalyLevel::Significant)
        .count();
    let elevated = anomalies
        .iter()
        .filter(|anomaly| anomaly.level == PerformanceAnomalyLevel::Elevated)
        .count();

    let summary = match overall {
        PerformanceAnomalyLevel::Normal =>
            "Current performance is consistent with the recent local baseline.".to_owned(),
        PerformanceAnomalyLevel::Elevated => format!(
            "{elevated} performance signal{} elevated relative to the recent local baseline.",
            if elevated == 1 { " is" } else { "s are" }
        ),
        PerformanceAnomalyLevel::Significant => format!(
            "{significant} performance signal{} significantly different from the recent local baseline.",
            if significant == 1 { " is" } else { "s are" }
        ),
    };

    anomalies.shrink_to_fit();
    PerformanceAnomalyReport {
        overall,
        anomalies,
        summary,
    }
}

fn percent_anomaly(
    metric: PerformanceMetric,
    current: f64,
    baseline: f64,
    deviation: f64,
    label: &str,
) -> PerformanceAnomaly {
    let relative = relative_change(current, baseline);
    let level = if deviation.abs() >= PERCENT_SIGNIFICANT_DELTA || relative >= RELATIVE_SIGNIFICANT {
        PerformanceAnomalyLevel::Significant
    } else if deviation.abs() >= PERCENT_ELEVATED_DELTA || relative >= RELATIVE_ELEVATED {
        PerformanceAnomalyLevel::Elevated
    } else {
        PerformanceAnomalyLevel::Normal
    };
    let direction = if deviation >= 0.0 { "above" } else { "below" };
    let explanation = if level == PerformanceAnomalyLevel::Normal {
        format!("{label} is consistent with the recent local baseline.")
    } else {
        format!(
            "{label} is {:.1} percentage points {direction} the recent local baseline ({baseline:.1}%).",
            deviation.abs()
        )
    };
    PerformanceAnomaly {
        metric,
        level,
        current_value: current,
        baseline_value: baseline,
        deviation,
        explanation,
    }
}

fn io_anomaly(
    metric: PerformanceMetric,
    current: f64,
    baseline: f64,
    deviation: f64,
    label: &str,
) -> PerformanceAnomaly {
    let relative = relative_change(current, baseline);
    let absolute = deviation.abs();
    let level = if absolute >= IO_SIGNIFICANT_BYTES_PER_SECOND || relative >= RELATIVE_SIGNIFICANT {
        PerformanceAnomalyLevel::Significant
    } else if absolute >= IO_ELEVATED_BYTES_PER_SECOND || relative >= RELATIVE_ELEVATED {
        PerformanceAnomalyLevel::Elevated
    } else {
        PerformanceAnomalyLevel::Normal
    };
    let direction = if deviation >= 0.0 { "higher" } else { "lower" };
    let explanation = if level == PerformanceAnomalyLevel::Normal {
        format!("{label} is consistent with the recent local baseline.")
    } else {
        format!(
            "{label} is {direction} than the recent local baseline by {}.",
            format_rate(absolute)
        )
    };
    PerformanceAnomaly {
        metric,
        level,
        current_value: current,
        baseline_value: baseline,
        deviation,
        explanation,
    }
}

fn count_anomaly(
    metric: PerformanceMetric,
    current: usize,
    baseline: usize,
    deviation: usize,
    label: &str,
    elevated_threshold: usize,
    significant_threshold: usize,
) -> PerformanceAnomaly {
    let level = if deviation >= significant_threshold {
        PerformanceAnomalyLevel::Significant
    } else if deviation >= elevated_threshold {
        PerformanceAnomalyLevel::Elevated
    } else {
        PerformanceAnomalyLevel::Normal
    };
    let direction = if current >= baseline { "higher" } else { "lower" };
    let explanation = if level == PerformanceAnomalyLevel::Normal {
        format!("{label} is consistent with the recent local baseline.")
    } else {
        format!(
            "{label} is {direction} than the recent local baseline by {deviation}.",
        )
    };
    PerformanceAnomaly {
        metric,
        level,
        current_value: current as f64,
        baseline_value: baseline as f64,
        deviation: deviation as f64,
        explanation,
    }
}

fn relative_change(current: f64, baseline: f64) -> f64 {
    if baseline.abs() < f64::EPSILON {
        if current.abs() < f64::EPSILON { 0.0 } else { 1.0 }
    } else {
        (current - baseline).abs() / baseline.abs()
    }
}

fn format_rate(bytes_per_second: f64) -> String {
    const UNITS: [&str; 4] = ["B/s", "KB/s", "MB/s", "GB/s"];
    let mut value = bytes_per_second;
    let mut index = 0;
    while value >= 1024.0 && index < UNITS.len() - 1 {
        value /= 1024.0;
        index += 1;
    }
    format!("{value:.1} {}", UNITS[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use monitoring::{NetworkRate, PerformanceBaseline, PerformanceDeviation};

    fn snapshot(
        cpu: f64,
        memory: f64,
        read: f64,
        write: f64,
        processes: usize,
        running: usize,
        baseline: Option<PerformanceBaseline>,
        deviation: Option<PerformanceDeviation>,
    ) -> MonitorSnapshot {
        MonitorSnapshot {
            timestamp_ms: 1,
            cpu_percent: cpu,
            memory_percent: memory,
            swap_percent: 0.0,
            network: vec![NetworkRate {
                name: "lo".into(),
                rx_bytes_per_second: 0.0,
                tx_bytes_per_second: 0.0,
            }],
            storage_read_bytes_per_second: read,
            storage_write_bytes_per_second: write,
            process_count: processes,
            running_processes: running,
            baseline,
            deviation,
        }
    }

    fn baseline() -> PerformanceBaseline {
        PerformanceBaseline {
            cpu_percent: 40.0,
            memory_percent: 50.0,
            storage_read_bytes_per_second: 2_000_000.0,
            storage_write_bytes_per_second: 2_000_000.0,
            process_count: 100,
            running_processes: 5,
        }
    }

    #[test]
    fn no_baseline_returns_collection_message() {
        let report = explain(&snapshot(40.0, 50.0, 0.0, 0.0, 100, 5, None, None));
        assert_eq!(report.overall, PerformanceAnomalyLevel::Normal);
        assert!(report.anomalies.is_empty());
        assert!(report.summary.contains("Collecting"));
    }

    #[test]
    fn normal_metrics_are_explained_as_normal() {
        let base = baseline();
        let report = explain(&snapshot(
            42.0,
            51.0,
            2_100_000.0,
            1_900_000.0,
            102,
            6,
            Some(base),
            None,
        ));
        assert_eq!(report.overall, PerformanceAnomalyLevel::Normal);
        assert!(report.anomalies.iter().all(|item| item.level == PerformanceAnomalyLevel::Normal));
    }

    #[test]
    fn elevated_cpu_is_reported() {
        let base = baseline();
        let report = explain(&snapshot(
            55.0,
            50.0,
            2_000_000.0,
            2_000_000.0,
            100,
            5,
            Some(base),
            None,
        ));
        let cpu = report.anomalies.iter().find(|item| item.metric == PerformanceMetric::Cpu).unwrap();
        assert_eq!(cpu.level, PerformanceAnomalyLevel::Elevated);
        assert!(cpu.explanation.contains("percentage points"));
    }

    #[test]
    fn significant_io_is_reported() {
        let base = baseline();
        let report = explain(&snapshot(
            40.0,
            50.0,
            20_000_000.0,
            2_000_000.0,
            100,
            5,
            Some(base),
            None,
        ));
        let read = report.anomalies.iter().find(|item| item.metric == PerformanceMetric::StorageRead).unwrap();
        assert_eq!(read.level, PerformanceAnomalyLevel::Significant);
    }

    #[test]
    fn process_deviation_is_reported() {
        let base = baseline();
        let report = explain(&snapshot(
            40.0,
            50.0,
            2_000_000.0,
            2_000_000.0,
            160,
            5,
            Some(base),
            None,
        ));
        let processes = report.anomalies.iter().find(|item| item.metric == PerformanceMetric::ProcessCount).unwrap();
        assert_eq!(processes.level, PerformanceAnomalyLevel::Significant);
        assert!(processes.explanation.contains("60"));
    }
}
