use std::borrow::Cow;

use reqwest::Method;

pub enum RouteInfo {
    Authenticate,
}

pub enum Route {
    Authenticate,
}

impl RouteInfo {
    pub fn deconstruct(&self) -> (Method, Route, Cow<'_, str>) {
        match *self {
            RouteInfo::Authenticate => (
                Method::POST,
                Route::Authenticate,
                Cow::from(Route::authenticate()),
            ),
        }
    }
}

impl Route {
    pub fn authenticate() -> &'static str {
        "/auth"
    }
}
