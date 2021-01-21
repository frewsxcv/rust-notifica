//! Small, cross-platform Rust crate for sending system notifications
//!
//! ## Examples
//!
//! ```rust
//! notifica::notify("Hello", "World! 🌍");
//! ```


#[cfg(target_os = "macos")]
extern crate mac_notification_sys;

#[cfg(target_os = "macos")]
use mac_notification_sys::error::{ApplicationError, NotificationError};

#[cfg(target_os = "linux")]
extern crate notify_rust;

#[cfg(target_os = "linux")]
use notify_rust::error::Error as LError;

#[cfg(target_os = "windows")]
use winrt::Error as WError;

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

trait Platform {
    fn setup() -> Self;
    fn notify(msg_title: &str, msg_body: &str) -> Result<(), ErrorRepr>;
}

#[derive(Debug)]
pub struct Error(ErrorRepr);

#[derive(Debug)]
enum ErrorRepr {
    #[cfg(target_os = "linux")]
    Linux(LError),
    #[cfg(target_os = "macos")]
    MacOs(MacOsError),
    #[cfg(target_os = "windows")]
    Windows(WError),
}

impl StdError for Error {}
impl StdError for ErrorRepr {}

#[cfg(target_os = "macos")]
#[derive(Debug)]
enum MacOsError {
    AppErr(ApplicationError),
    NotErr(NotificationError),
}

#[cfg(target_os = "macos")]
impl Display for MacOsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MacOsError::AppErr(e) => Display::fmt(e, f),
            MacOsError::NotErr(e) => Display::fmt(e, f),
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(fmt)
    }
}

impl Display for ErrorRepr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            #[cfg(target_os = "linux")]
            ErrorRepr::Linux(e) => write!(fmt, "{}", e),
            #[cfg(target_os = "macos")]
            ErrorRepr::MacOs(e) => write!(fmt, "{}", e),
            #[cfg(target_os = "windows")]
            ErrorRepr::Windows(e) => write!(fmt, "{:?}", e),
        }
    }
}

#[cfg(target_os = "macos")]
impl From<ApplicationError> for MacOsError {
    fn from(err: ApplicationError) -> Self {
        MacOsError::AppErr(err)
    }
}

#[cfg(target_os = "macos")]
impl From<ApplicationError> for ErrorRepr {
    fn from(err: ApplicationError) -> Self {
        ErrorRepr::MacOs(err.into())
    }
}

#[cfg(target_os = "macos")]
impl From<NotificationError> for MacOsError {
    fn from(err: NotificationError) -> Self {
        MacOsError::NotErr(err)
    }
}

#[cfg(target_os = "macos")]
impl From<NotificationError> for ErrorRepr {
    fn from(err: NotificationError) -> Self {
        ErrorRepr::MacOs(err.into())
    }
}

#[cfg(target_os = "linux")]
impl From<LError> for ErrorRepr {
    fn from(err: LError) -> Self {
        ErrorRepr::Linux(err)
    }
}

#[cfg(target_os = "windows")]
impl From<WError> for ErrorRepr {
    fn from(err: WError) -> Self {
        ErrorRepr::Windows(err)
    }
}

#[cfg(target_os = "windows")]
struct Windows;

#[cfg(target_os = "windows")]
impl Platform for Windows {
    fn setup() -> Self {
        Windows
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), ErrorRepr> {
        use winrt::windows::data::xml::dom::*;
        use winrt::windows::ui::notifications::*;
        use winrt::*;
        let toast_xml =
            ToastNotificationManager::get_template_content(ToastTemplateType::ToastText02)?.unwrap();
        let toast_text_elements =
            toast_xml.get_elements_by_tag_name(&FastHString::new("text"))?.unwrap();

        toast_text_elements.item(0)?.unwrap().append_child(
            &*toast_xml
                .create_text_node(&FastHString::from(msg_title))?.unwrap()
                .query_interface::<IXmlNode>().unwrap(),
        )?;
        toast_text_elements.item(1)?.unwrap().append_child(
            &*toast_xml
                .create_text_node(&FastHString::from(msg_body))?.unwrap()
                .query_interface::<IXmlNode>().unwrap(),
        )?;

        let toast = ToastNotification::create_toast_notification(&*toast_xml)?;
        ToastNotificationManager::create_toast_notifier_with_id(&FastHString::new(
            "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
        ))?.unwrap()
            .show(&*toast)?;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
struct MacOs;

#[cfg(target_os = "macos")]
impl Platform for MacOs {
    fn setup() -> Self {
        MacOs
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), ErrorRepr> {
        let bundle = mac_notification_sys::get_bundle_identifier("Script Editor").unwrap();
        mac_notification_sys::set_application(&bundle).unwrap();
        mac_notification_sys::send_notification(msg_title, &None, msg_body, &None).unwrap();
        Ok(())
    }
}

#[cfg(target_os = "linux")]
struct Linux;

#[cfg(target_os = "linux")]
impl Platform for Linux {
    fn setup() -> Self {
        Linux
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), ErrorRepr> {
        notify_rust::Notification::new()
            .summary(msg_title)
            .body(msg_body)
            .show()?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
type CurrPlatform = Windows;
#[cfg(target_os = "macos")]
type CurrPlatform = MacOs;
#[cfg(target_os = "linux")]
type CurrPlatform = Linux;

pub fn notify(msg_title: &str, msg_body: &str) -> Result<(), Error> {
    CurrPlatform::setup();
    CurrPlatform::notify(msg_title, msg_body).map_err(Error)?;
    Ok(())
}
