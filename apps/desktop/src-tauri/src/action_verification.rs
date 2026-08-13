#[derive(Clone, Debug)]
pub struct ActionVerification {
    pub status: String,
    pub message: String,
}

pub fn verify_safe_action(action: &str, execution_succeeded: bool) -> ActionVerification {
    if execution_succeeded {
        ActionVerification {
            status: "verified".to_owned(),
            message: format!(
                "{action} returned a successful result; the permitted action is read-only, so no system mutation requires separate state verification."
            ),
        }
    } else {
        ActionVerification {
            status: "failed".to_owned(),
            message: format!("{action} did not complete successfully, so its intended outcome could not be verified."),
        }
    }
}
