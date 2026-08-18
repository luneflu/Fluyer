import { IconType } from '$lib/ui/icon/types';

import HelpCircleOutline from 'svelte-material-icons/HelpCircleOutline.svelte';
import PlayCircleOutline from 'svelte-material-icons/PlayCircleOutline.svelte';
import PauseCircleOutline from 'svelte-material-icons/PauseCircleOutline.svelte';
import SkipPreviousCircleOutline from 'svelte-material-icons/SkipPreviousCircleOutline.svelte';
import SkipNextCircleOutline from 'svelte-material-icons/SkipNextCircleOutline.svelte';
import MusicNote from 'svelte-material-icons/MusicNote.svelte';
import ArrowULeftTop from 'svelte-material-icons/ArrowULeftTop.svelte';
import VolumeHigh from 'svelte-material-icons/VolumeHigh.svelte';
import VolumeOff from 'svelte-material-icons/VolumeOff.svelte';
import CloseCircleOutline from 'svelte-material-icons/CloseCircleOutline.svelte';
import Delete from 'svelte-material-icons/Delete.svelte';
import Cog from 'svelte-material-icons/Cog.svelte';
import Magnify from 'svelte-material-icons/Magnify.svelte';
import PlaylistPlus from 'svelte-material-icons/PlaylistPlus.svelte';
import Broom from 'svelte-material-icons/Broom.svelte';
import Repeat from 'svelte-material-icons/Repeat.svelte';
import RepeatOnce from 'svelte-material-icons/RepeatOnce.svelte';
import Shuffle from 'svelte-material-icons/Shuffle.svelte';
import Fullscreen from 'svelte-material-icons/Fullscreen.svelte';
import DotsGrid from 'svelte-material-icons/DotsGrid.svelte';
import Lock from 'svelte-material-icons/Lock.svelte';
import FileDocumentOutline from 'svelte-material-icons/FileDocumentOutline.svelte';
import TuneVerticalVariant from 'svelte-material-icons/TuneVerticalVariant.svelte';
import Poll from 'svelte-material-icons/Poll.svelte';
import Folder from 'svelte-material-icons/Folder.svelte';
import SortAscending from 'svelte-material-icons/SortAscending.svelte';
import SortDescending from 'svelte-material-icons/SortDescending.svelte';
import GridLarge from 'svelte-material-icons/GridLarge.svelte';
import Album from 'svelte-material-icons/Album.svelte';
import PlaylistMusicOutline from 'svelte-material-icons/PlaylistMusicOutline.svelte';
import Check from 'svelte-material-icons/Check.svelte';
import Close from 'svelte-material-icons/Close.svelte';
import Image from 'svelte-material-icons/Image.svelte';
import Menu from 'svelte-material-icons/Menu.svelte';
import PlaylistMusic from 'svelte-material-icons/PlaylistMusic.svelte';

const iconRegistryMaterial = {
	[IconType.Unknown]: HelpCircleOutline,
	[IconType.Play]: PlayCircleOutline,
	[IconType.Pause]: PauseCircleOutline,
	[IconType.Previous]: SkipPreviousCircleOutline,
	[IconType.Next]: SkipNextCircleOutline,
	[IconType.Playing]: MusicNote,
	[IconType.Note]: MusicNote,
	[IconType.PlayBack]: ArrowULeftTop,
	[IconType.Back]: ArrowULeftTop,
	[IconType.AlbumBack]: ArrowULeftTop,
	[IconType.Speaker]: VolumeHigh,
	[IconType.Mute]: VolumeOff,
	[IconType.Remove]: CloseCircleOutline,
	[IconType.Trash]: Delete,
	[IconType.TrashRed]: Delete,
	[IconType.Settings]: Cog,
	[IconType.Search]: Magnify,
	[IconType.QueueMusic]: PlaylistPlus,
	[IconType.CleanQueue]: Broom,
	[IconType.RepeatNone]: Repeat,
	[IconType.RepeatPlayNone]: Repeat,
	[IconType.Repeat]: Repeat,
	[IconType.RepeatOne]: RepeatOnce,
	[IconType.Shuffle]: Shuffle,
	[IconType.Fullscreen]: Fullscreen,
	[IconType.DragOn]: DotsGrid,
	[IconType.DragOff]: Lock,
	[IconType.SaveLog]: FileDocumentOutline,
	[IconType.Equalizer]: TuneVerticalVariant,
	[IconType.Close]: CloseCircleOutline,
	[IconType.Visualizer]: Poll,
	[IconType.Folder]: Folder,
	[IconType.SortAsc]: SortAscending,
	[IconType.SortDesc]: SortDescending,
	[IconType.MusicListTypeAll]: GridLarge,
	[IconType.MusicListTypeAlbum]: Album,
	[IconType.MusicListTypeMusic]: MusicNote,
	[IconType.MusicListTypeFolder]: Folder,
	[IconType.MusicListTypePlaylist]: PlaylistMusicOutline,
	[IconType.PlaylistAdd]: PlaylistPlus,
	[IconType.Check]: Check,
	[IconType.Cancel]: Close,
	[IconType.Image]: Image,
	[IconType.Menu]: Menu,
	[IconType.Queue]: PlaylistMusic
};

export default iconRegistryMaterial;
