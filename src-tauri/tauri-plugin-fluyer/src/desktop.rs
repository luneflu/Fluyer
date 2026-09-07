use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Fluyer<R>> {
    Ok(Fluyer(app.clone()))
}

/// Access to the fluyer APIs.
pub struct Fluyer<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Fluyer<R> {
    pub fn toast(&self, _value: String) -> crate::Result<()> {
        Ok(())
    }

    pub fn navigation_bar_height_get(&self) -> crate::Result<NavigationBarHeight> {
        Ok(NavigationBarHeight::default())
    }

    pub fn get_navigation_bar_size(&self) -> crate::Result<NavigationBarSize> {
        Ok(NavigationBarSize::default())
    }

    pub fn status_bar_height_get(&self) -> crate::Result<StatusBarHeight> {
        Ok(StatusBarHeight::default())
    }

    pub fn check_permissions(&self) -> crate::Result<Option<PermissionStatus>> {
        Ok(None)
    }

    pub fn request_permissions(
        &self,
        _permissions: Option<Vec<PermissionType>>,
    ) -> crate::Result<PermissionStatus> {
        Ok(PermissionStatus::default())
    }

    pub fn restart_app(&self) -> crate::Result<()> {
        Ok(())
    }

    pub fn get_sdk_version(&self) -> crate::Result<SdkVersion> {
        Ok(SdkVersion::default())
    }

    pub fn navigation_bar_visibility_set(&self, _: bool) -> crate::Result<()> {
        Ok(())
    }

    pub fn android_pick_folder<F: Fn(WatcherPickFolder) + Send + Sync + 'static>(
        &self,
        _callback: F,
    ) -> crate::Result<WatchPickFolderResponse> {
        Ok(WatchPickFolderResponse { value: false })
    }

    pub fn init_media_control<F: Fn(MediaControlEvent) + Send + Sync + 'static>(
        &self,
        _callback: F,
    ) -> crate::Result<()> {
        Ok(())
    }

    pub fn update_media_control(
        &self,
        _title: String,
        _artist: String,
        _album: String,
        _duration: u64,
        _artwork_path: Option<String>,
    ) -> crate::Result<()> {
        Ok(())
    }

    pub fn set_media_control_state(&self, _is_playing: bool, _position: u64) -> crate::Result<()> {
        Ok(())
    }
}

impl<R: Runtime> Fluyer<R> {}
