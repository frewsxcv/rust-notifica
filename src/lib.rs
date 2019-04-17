#[cfg(target_os = "macos")]
extern crate mac_notification_sys;

#[cfg(target_os = "macos")]
use mac_notification_sys::error::{ApplicationError, NotificationError};

#[cfg(target_os = "linux")]
extern crate notify_rust;

#[cfg(target_os = "linux")]
use notify_rust::error::Error as LError;

#[cfg(target_os = "windows")]
extern crate winrt;

#[cfg(target_os = "windows")]
use winrt::Error as WError;

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

trait Platform {
    fn setup() -> Self;
    fn notify(msg_title: &str, msg_body: &str) -> Result<(), Error>;
    fn teardown(self);
}

#[derive(Debug)]
enum Error {
    #[cfg(target_os = "linux")]
    Linux(LError),
    #[cfg(target_os = "macos")]
    MacOs(MacOsError),
    #[cfg(target_os = "windows")]
    Windows(WError),
}

impl StdError for Error {}

#[cfg(target_os = "macos")]
enum MacOsError {
    AppErr(ApplicationError),
    NotErr(NotificationError),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            #[cfg(target_os = "linux")]
            Error::Linux(e) => write!(fmt, "{}", e),
            #[cfg(target_os = "macos")]
            Error::MacOs => write!(fmt, "MacOs Error"),
            #[cfg(target_os = "windows")]
            Error::Windows => write!(fmt, "Windows Error"),
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
impl From<ApplicationError> for Error {
    fn from(err: ApplicationError) -> Self {
        Error::MacOs(err.into())
    }
}

#[cfg(target_os = "macos")]
impl From<NotificationError> for MacOsError {
    fn from(err: NotificationError) -> Self {
        MacOsError::NotErr(err)
    }
}

#[cfg(target_os = "macos")]
impl From<NotificationError> for Error {
    fn from(err: NotificationError) -> Self {
        Error::MacOs(err.into())
    }
}

#[cfg(target_os = "linux")]
impl From<LError> for Error {
    fn from(err: LError) -> Self {
        Error::Linux(err)
    }
}

#[cfg(target_os = "windows")]
impl From<WError> for Error {
    fn from(err: WError) -> Self {
        Error::Windows(err)
    }
}

#[cfg(target_os = "windows")]
struct Windows(winrt::RuntimeContext);

#[cfg(target_os = "windows")]
impl Platform for Windows {
    fn setup() -> Self {
        Windows(winrt::RuntimeContext::init())
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), Error> {
        use winrt::windows::data::xml::dom::*;
        use winrt::windows::ui::notifications::*;
        use winrt::*;
        let toast_xml =
            ToastNotificationManager::get_template_content(ToastTemplateType::ToastText02)??;
        let toast_text_elements =
            toast_xml.get_elements_by_tag_name(&FastHString::new("text"))??;

        toast_text_elements.item(0)??.append_child(
            &*toast_xml
                .create_text_node(&FastHString::from(msg_title))??
                .query_interface::<IXmlNode>()?,
        )?;
        toast_text_elements.item(1)??.append_child(
            &*toast_xml
                .create_text_node(&FastHString::from(msg_body))??
                .query_interface::<IXmlNode>()?,
        )?;

        let toast = ToastNotification::create_toast_notification(&*toast_xml)?;
        ToastNotificationManager::create_toast_notifier_with_id(&FastHString::new(
            "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
        ))??
        .show(&*toast)?;
        Ok(())
    }

    fn teardown(self) {
        self.0.uninit();
    }
}

#[cfg(target_os = "macos")]
struct MacOs;

#[cfg(target_os = "macos")]
impl Platform for MacOs {
    fn setup() -> Self {
        MacOs
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), Error> {
        let bundle = mac_notification_sys::get_bundle_identifier("Script Editor")?;
        mac_notification_sys::set_application(&bundle)?;
        mac_notification_sys::send_notification(msg_title, &None, msg_body, &None)?;
    }

    fn teardown(self) {}
}

#[cfg(target_os = "linux")]
struct Linux;

#[cfg(target_os = "linux")]
impl Platform for Linux {
    fn setup() -> Self {
        Linux
    }

    fn notify(msg_title: &str, msg_body: &str) -> Result<(), Error> {
        notify_rust::Notification::new()
            .summary(msg_title)
            .body(msg_body)
            .show()?;
        Ok(())
    }

    fn teardown(self) {}
}

#[cfg(target_os = "windows")]
type CurrPlatform = Windows;
#[cfg(target_os = "macos")]
type CurrPlatform = MacOs;
#[cfg(target_os = "linux")]
type CurrPlatform = Linux;

pub fn notify(msg_title: &str, msg_body: &str) {
    let p = CurrPlatform::setup();
    CurrPlatform::notify(msg_title, msg_body);
    p.teardown();
}
