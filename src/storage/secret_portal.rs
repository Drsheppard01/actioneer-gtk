use gio::glib::{self, VariantTy};
use gio::prelude::*;
use gio::{self, DBusCallFlags, DBusProxyFlags};
use thiserror::Error;
use tracing::debug;

const PORTAL_BUS_NAME: &str = "org.freedesktop.portal.Desktop";
const PORTAL_OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";
const INTROSPECT_INTERFACE: &str = "org.freedesktop.DBus.Introspectable";
const SECRET_INTERFACE: &str = "org.freedesktop.portal.Secret";
const DETECTION_ENV: &str = "ACTIONEER_ENABLE_SECRET_PORTAL";
const INTROSPECT_TIMEOUT_MS: i32 = 5_000;

#[derive(Debug, Error)]
pub enum PortalDetectionError {
    #[error("failed to connect to session bus: {0}")]
    Bus(#[from] glib::Error),

    #[error("portal introspection response was not a string")]
    UnexpectedIntrospection,
}

/// Returns `true` when experimental portal detection has been requested via env var.
pub fn detection_enabled() -> bool {
    std::env::var(DETECTION_ENV)
        .ok()
        .map(|value| parse_flag(&value))
        .unwrap_or(false)
}

/// Inspect the desktop portal to determine if the Secret interface is advertised.
pub fn secret_portal_available() -> Result<bool, PortalDetectionError> {
    let connection = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)?;

    let proxy = gio::DBusProxy::new_sync(
        &connection,
        DBusProxyFlags::DO_NOT_AUTO_START | DBusProxyFlags::DO_NOT_LOAD_PROPERTIES,
        None::<&gio::DBusInterfaceInfo>,
        Some(PORTAL_BUS_NAME),
        PORTAL_OBJECT_PATH,
        INTROSPECT_INTERFACE,
        None::<&gio::Cancellable>,
    )?;

    let xml_variant = proxy.call_sync(
        "Introspect",
        None::<&glib::Variant>,
        DBusCallFlags::NONE,
        INTROSPECT_TIMEOUT_MS,
        None::<&gio::Cancellable>,
    )?;

    let xml_variant = if xml_variant.is_type(&VariantTy::STRING) {
        xml_variant
    } else {
        xml_variant.child_value(0)
    };
    let xml = xml_variant
        .str()
        .ok_or(PortalDetectionError::UnexpectedIntrospection)?;
    let available = xml.contains(SECRET_INTERFACE);

    if available {
        debug!("Found {SECRET_INTERFACE} via portal introspection");
    } else {
        debug!("{SECRET_INTERFACE} missing from portal introspection");
    }

    Ok(available)
}

fn parse_flag(value: &str) -> bool {
    match value {
        "1" | "true" | "TRUE" | "True" | "yes" | "YES" | "Yes" | "on" | "ON" | "On" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_truthy_flags() {
        for value in [
            "1", "true", "TRUE", "True", "yes", "YES", "Yes", "on", "ON", "On",
        ] {
            assert!(
                parse_flag(value),
                "Expected '{value}' to be recognized as true"
            );
        }
        assert!(!parse_flag("0"));
        assert!(!parse_flag("false"));
        assert!(!parse_flag(""));
    }

    #[test]
    #[ignore = "Requires org.freedesktop.portal.Secret to be available on the host"]
    fn detects_secret_portal_when_available() {
        let available = secret_portal_available()
            .expect("Failed to query the desktop portal for Secret support");
        assert!(available, "Secret portal interface not found");
    }
}
