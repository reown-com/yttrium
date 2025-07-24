use leptos::prelude::*;
use thaw::{Toast, ToastIntent, ToastOptions, ToastTitle, ToasterInjection};

pub fn show_success_toast(toaster: ToasterInjection, title: String) {
    show_toast(
        toaster,
        title,
        ToastOptions::default().with_intent(ToastIntent::Success),
    );
}

pub fn show_error_toast(toaster: ToasterInjection, title: String) {
    show_toast(
        toaster,
        title,
        ToastOptions::default().with_intent(ToastIntent::Error),
    );
}

pub fn show_toast(
    toaster: ToasterInjection,
    title: String,
    options: ToastOptions,
) {
    toaster.dispatch_toast(
        move || {
            view! {
                <Toast>
                    <ToastTitle>{title}</ToastTitle>
                </Toast>
            }
        },
        options,
    );
}
