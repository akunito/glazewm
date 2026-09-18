use tracing::info;
use wm_common::{DisplayState, HideMethod, WindowState};
use wm_platform::NativeWindow;

use crate::{
  commands::window::{manage_window, update_window_state},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    info!("Window shown: {window}");

    // A window that covers its monitor (a borderless game) can have been
    // demoted from fullscreen by a transient size while it was starting or
    // being hidden. Re-check now that it's visible again, otherwise it stays
    // floating forever and the taskbar is never pushed below it.
    if let Some(workspace) = window.workspace() {
      if !matches!(window.state(), WindowState::Fullscreen(_))
        && window.should_fullscreen(&workspace).unwrap_or(false)
      {
        info!("Window covers its monitor, restoring fullscreen: {window}");

        let fullscreen_state = config
          .value
          .window_behavior
          .state_defaults
          .fullscreen
          .clone();

        update_window_state(
          window.clone(),
          WindowState::Fullscreen(fullscreen_state),
          state,
          config,
        )?;

        return Ok(());
      }
    }

    // Update display state if window is already managed.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Showing
    {
      window.set_display_state(DisplayState::Shown);
    } else {
      state.pending_sync.queue_container_to_redraw(window);
    }
  } else {
    // If the window is not managed, manage it.
    manage_window(native_window, None, state, config)?;
  }

  Ok(())
}
