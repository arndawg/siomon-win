use std::collections::HashMap;

/// Load sensor label overrides. Checks:
/// 1. Built-in board-specific labels (matched by board name from DMI)
/// 2. User overrides from config file (these take precedence)
pub fn load_labels(
    board_name: Option<&str>,
    user_labels: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut labels = HashMap::new();

    // Built-in board labels
    if let Some(board) = board_name {
        labels.extend(builtin_labels(board));
    }

    // User labels override built-ins
    labels.extend(user_labels.clone());

    labels
}

/// Read the board name from DMI sysfs.
pub fn read_board_name() -> Option<String> {
    crate::platform::sysfs::read_string_optional(std::path::Path::new(
        "/sys/class/dmi/id/board_name",
    ))
}

fn builtin_labels(board: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();

    // ASUS WRX90E-SAGE SE (nct6798)
    if board.contains("WRX90E") {
        m.insert("hwmon/nct6798/in0".into(), "Vcore".into());
        m.insert("hwmon/nct6798/in1".into(), "VIN1".into());
        m.insert("hwmon/nct6798/in2".into(), "+3.3V".into());
        m.insert("hwmon/nct6798/in3".into(), "+3.3V Standby".into());
        m.insert("hwmon/nct6798/in4".into(), "VIN4".into());
        m.insert("hwmon/nct6798/in5".into(), "VIN5".into());
        m.insert("hwmon/nct6798/in6".into(), "VIN6".into());
        m.insert("hwmon/nct6798/in7".into(), "+3.3V AUX".into());
        m.insert("hwmon/nct6798/in8".into(), "Vbat".into());
        m.insert("hwmon/nct6798/temp1".into(), "SYSTIN".into());
        m.insert("hwmon/nct6798/temp2".into(), "CPUTIN".into());
        m.insert("hwmon/nct6798/temp3".into(), "AUXTIN0".into());
        m.insert("hwmon/nct6798/fan1".into(), "CPU Fan".into());
        m.insert("hwmon/nct6798/fan2".into(), "Chassis Fan 1".into());
        m.insert("hwmon/nct6798/fan3".into(), "Chassis Fan 2".into());
        m.insert("hwmon/nct6798/fan4".into(), "Chassis Fan 3".into());
        m.insert("hwmon/nct6798/fan5".into(), "Chassis Fan 4".into());
        m.insert("hwmon/nct6798/fan6".into(), "Chassis Fan 5".into());
        m.insert("hwmon/nct6798/fan7".into(), "AIO Pump".into());
    }

    // ASUS ROG CROSSHAIR X670E (NCT6798D)
    if board.contains("CROSSHAIR") && board.contains("X670") {
        insert_asus_am5_nct6798(&mut m);
        m.insert("hwmon/nct6798/fan2".into(), "CPU OPT".into());
    }

    // ASUS ROG STRIX X670E / B650E (NCT6798D)
    if board.contains("STRIX") && (board.contains("X670") || board.contains("B650")) {
        insert_asus_am5_nct6798(&mut m);
        m.insert("hwmon/nct6798/fan2".into(), "Chassis Fan 1".into());
        m.insert("hwmon/nct6798/fan3".into(), "Chassis Fan 2".into());
        m.insert("hwmon/nct6798/fan4".into(), "Chassis Fan 3".into());
    }

    // ASUS TUF GAMING X670E / B650 (NCT6798D)
    if board.contains("TUF") && (board.contains("X670") || board.contains("B650")) {
        insert_asus_am5_nct6798(&mut m);
        m.insert("hwmon/nct6798/fan2".into(), "Chassis Fan 1".into());
        m.insert("hwmon/nct6798/fan3".into(), "Chassis Fan 2".into());
    }

    // ASUS PRIME X670E / B650 (NCT6798D)
    if board.contains("PRIME") && (board.contains("X670") || board.contains("B650")) {
        insert_asus_am5_nct6798(&mut m);
        m.insert("hwmon/nct6798/fan2".into(), "Chassis Fan 1".into());
    }

    // ASUS ProArt X670E (NCT6798D)
    if board.contains("PROART") && board.contains("X670") {
        insert_asus_am5_nct6798(&mut m);
        m.insert("hwmon/nct6798/fan2".into(), "Chassis Fan 1".into());
        m.insert("hwmon/nct6798/fan3".into(), "Chassis Fan 2".into());
    }

    m
}

/// Common sensor labels shared across ASUS AM5 boards with NCT6798D.
fn insert_asus_am5_nct6798(m: &mut HashMap<String, String>) {
    m.insert("hwmon/nct6798/in0".into(), "Vcore".into());
    m.insert("hwmon/nct6798/in1".into(), "+5V".into());
    m.insert("hwmon/nct6798/in2".into(), "AVCC".into());
    m.insert("hwmon/nct6798/in3".into(), "+3.3V".into());
    m.insert("hwmon/nct6798/in4".into(), "+12V".into());
    m.insert("hwmon/nct6798/in7".into(), "+3.3V AUX".into());
    m.insert("hwmon/nct6798/in8".into(), "Vbat".into());
    m.insert("hwmon/nct6798/temp1".into(), "SYSTIN".into());
    m.insert("hwmon/nct6798/temp2".into(), "CPUTIN".into());
    m.insert("hwmon/nct6798/temp3".into(), "AUXTIN0".into());
    m.insert("hwmon/nct6798/fan1".into(), "CPU Fan".into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_labels_wrx90e() {
        let labels = builtin_labels("Pro WS WRX90E-SAGE SE");
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "Vcore");
        assert_eq!(labels.get("hwmon/nct6798/fan7").unwrap(), "AIO Pump");
    }

    #[test]
    fn test_builtin_labels_crosshair_x670() {
        let labels = builtin_labels("ROG CROSSHAIR X670E HERO");
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "Vcore");
        assert_eq!(labels.get("hwmon/nct6798/in4").unwrap(), "+12V");
        assert_eq!(labels.get("hwmon/nct6798/fan1").unwrap(), "CPU Fan");
        assert_eq!(labels.get("hwmon/nct6798/fan2").unwrap(), "CPU OPT");
    }

    #[test]
    fn test_builtin_labels_strix_x670e() {
        let labels = builtin_labels("ROG STRIX X670E-E GAMING WIFI");
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "Vcore");
        assert_eq!(labels.get("hwmon/nct6798/in1").unwrap(), "+5V");
        assert_eq!(labels.get("hwmon/nct6798/fan1").unwrap(), "CPU Fan");
        assert_eq!(labels.get("hwmon/nct6798/fan2").unwrap(), "Chassis Fan 1");
    }

    #[test]
    fn test_builtin_labels_tuf_b650() {
        let labels = builtin_labels("TUF GAMING B650-PLUS WIFI");
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "Vcore");
        assert_eq!(labels.get("hwmon/nct6798/fan1").unwrap(), "CPU Fan");
        assert_eq!(labels.get("hwmon/nct6798/fan2").unwrap(), "Chassis Fan 1");
    }

    #[test]
    fn test_builtin_labels_prime_x670e() {
        let labels = builtin_labels("PRIME X670E-PRO WIFI");
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "Vcore");
        assert_eq!(labels.get("hwmon/nct6798/fan1").unwrap(), "CPU Fan");
    }

    #[test]
    fn test_builtin_labels_unknown_board() {
        let labels = builtin_labels("Some Unknown Board");
        assert!(labels.is_empty());
    }

    #[test]
    fn test_user_labels_override_builtin() {
        let mut user = HashMap::new();
        user.insert("hwmon/nct6798/in0".into(), "My Custom Vcore".into());

        let labels = load_labels(Some("WRX90E-SAGE SE"), &user);
        // User label takes precedence over the built-in "Vcore"
        assert_eq!(labels.get("hwmon/nct6798/in0").unwrap(), "My Custom Vcore");
        // Built-in labels for other sensors still present
        assert_eq!(labels.get("hwmon/nct6798/fan1").unwrap(), "CPU Fan");
    }

    #[test]
    fn test_load_labels_no_board() {
        let mut user = HashMap::new();
        user.insert("hwmon/coretemp/temp1".into(), "CPU Package".into());

        let labels = load_labels(None, &user);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels.get("hwmon/coretemp/temp1").unwrap(), "CPU Package");
    }
}
