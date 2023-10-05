//! A model for system specific state which can be accessed by any model or view.
use crate::{model::Model, prelude::Wrapper, window::WindowEvent};
use unic_langid::LanguageIdentifier;
use vizia_derive::Lens;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    DarkMode,
    LightMode,
}

use crate::{binding::Lens, context::EventContext, events::Event};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppTheme {
    System,
    BuiltIn(ThemeMode),
    // Custom(String),
}

/// A model for system specific state which can be accessed by any model or view.
#[derive(Lens)]
pub struct Environment {
    // The locale used for localization.
    pub locale: LanguageIdentifier,
    // The current application theme
    pub app_theme: AppTheme,
    // The current system theme
    pub sys_theme: Option<ThemeMode>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        let locale = sys_locale::get_locale().and_then(|l| l.parse().ok()).unwrap_or_default();

        Self { locale, app_theme: AppTheme::BuiltIn(ThemeMode::LightMode), sys_theme: None }
    }

    pub fn get_current_theme(&self) -> ThemeMode {
        match self.app_theme {
            AppTheme::System => self.sys_theme.unwrap_or_default(),
            AppTheme::BuiltIn(theme) => theme,
        }
    }
}

/// Events for setting the state in the [Environment].
pub enum EnvironmentEvent {
    /// Set the locale used for the whole application.
    SetLocale(LanguageIdentifier),
    /// Set the default theme mode.
    // TODO: add SetSysTheme event when the winit `set_theme` fixed.
    SetThemeMode(AppTheme),
    /// Reset the locale to use the system provided locale.
    UseSystemLocale,
    /// Alternate between dark and light theme modes.
    ToggleThemeMode,
}

impl Model for Environment {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, _| match event {
            EnvironmentEvent::SetLocale(locale) => {
                self.locale = locale.clone();
            }

            EnvironmentEvent::SetThemeMode(theme_mode) => {
                self.app_theme = theme_mode.to_owned();

                cx.set_theme_mode(self.get_current_theme());
                cx.reload_styles().unwrap();
            }

            EnvironmentEvent::UseSystemLocale => {
                self.locale =
                    sys_locale::get_locale().map(|l| l.parse().unwrap()).unwrap_or_default();
            }

            EnvironmentEvent::ToggleThemeMode => {
                let theme_mode = match self.get_current_theme() {
                    ThemeMode::DarkMode => ThemeMode::LightMode,
                    ThemeMode::LightMode => ThemeMode::DarkMode,
                };

                self.app_theme = AppTheme::BuiltIn(theme_mode);

                cx.set_theme_mode(theme_mode);
                cx.reload_styles().unwrap();
            }
        });

        event.map(|event, _| match event {
            WindowEvent::ThemeChanged(theme) => {
                self.sys_theme = Some(*theme);
                if self.app_theme == AppTheme::System {
                    cx.set_theme_mode(*theme);
                    cx.reload_styles().unwrap();
                }
            }
            _ => (),
        })
    }
}
