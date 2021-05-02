use cursive::views::Button;
use cursive::{traits::Boxable, View};
use log::{error, info};

use crate::{
    common::dom::{element::Element, Node},
    tui::{components::TextInputView, views::with_current_browser_view, BrowserView},
};

use super::RenderError;

pub fn render(_node: &Node, element: &Element) -> Result<Box<dyn View>, RenderError> {
    match element
        .attributes
        .get("type")
        .unwrap_or(&"".to_string())
        .as_str()
    {
        "text" => Ok(Box::new(
            TextInputView::new()
                .content(element.attributes.get("value").unwrap_or(&"".to_string()))
                .min_width(10)
                .max_width(10),
        )),
        "button" | "submit" => {
            let onclick = element
                .attributes
                .get("onclick")
                .unwrap_or(&"".to_string())
                .clone();

            Ok(Box::new(Button::new(
                element.attributes.get("value").unwrap_or(&"".to_string()),
                move |s| {
                    let result = with_current_browser_view(s, |b: &mut BrowserView| {
                        b.with_page_view_mut(|p| p.js_runtime.execute("(inline)", onclick.as_str()))
                    });
                    if result.is_none() {
                        error!("failed to run onclick event of button")
                    }
                    match result.unwrap().unwrap() {
                        Ok(message) => {
                            info!("succeeded to run javascript; {}", message);
                        }
                        Err(e) => {
                            error!(
                                "failed to run javascript; {}",
                                RenderError::JavaScriptError(e)
                            );
                        }
                    }
                },
            )))
        }
        t => {
            info!("unsupported input tag type {} found", t);
            Err(RenderError::UnsupportedInputTypeError {
                specified_type: t.to_string(),
            })
        }
    }
}