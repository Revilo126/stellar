//! Provides an api for functionality similar to bevy's,
//! As this was made in my spite of bevy. Not hate for their systems.

use crate::app::App;

pub trait Plugin: 'static {
    fn build(&self, app: &mut App);
}
