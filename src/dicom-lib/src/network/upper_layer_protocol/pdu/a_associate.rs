pub mod application_context;
pub mod presentation_context;
pub mod user_information;

pub use application_context::ApplicationContext;
pub use user_information::UserInformation;

pub(crate) const INVALID_ITEM_TYPE_ERROR_MESSAGE: &str = "Item-type が不正です";
pub(crate) const INVALID_ITEM_LENGTH_ERROR_MESSAGE: &str = "Item-length が不正です";
