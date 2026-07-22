use ikihs_core::Error;
use ikihs_core::theme::Theme;

pub struct TmThemeParser;

impl TmThemeParser {
    pub fn parse_xml(_xml: &str) -> Result<Theme, Error> {
        Err(Error::Theme("tmTheme parsing not yet implemented".into()))
    }
}
