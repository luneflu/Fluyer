import { IconType } from '$lib/ui/icon/types';
import Undo2 from '@lucide/svelte/icons/undo-2';
import BrushCleaning from '@lucide/svelte/icons/brush-cleaning';
import GripVertical from '@lucide/svelte/icons/grip-vertical';
import FileText from '@lucide/svelte/icons/file-text';
import Expand from '@lucide/svelte/icons/expand';
import Settings from '@lucide/svelte/icons/settings';
import Lock from '@lucide/svelte/icons/lock';
import Search from '@lucide/svelte/icons/search';
import Music2 from '@lucide/svelte/icons/music-2';
import Pause from '@lucide/svelte/icons/pause';
import Play from '@lucide/svelte/icons/play';
import HelpCircle from '@lucide/svelte/icons/help-circle';
import ListPlus from '@lucide/svelte/icons/list-plus';
import Repeat from '@lucide/svelte/icons/repeat';
import Repeat1 from '@lucide/svelte/icons/repeat-1';
import Shuffle from '@lucide/svelte/icons/shuffle';
import SkipBack from '@lucide/svelte/icons/skip-back';
import SkipForward from '@lucide/svelte/icons/skip-forward';
import Volume2 from '@lucide/svelte/icons/volume-2';
import VolumeX from '@lucide/svelte/icons/volume-x';
import Trash2 from '@lucide/svelte/icons/trash-2';
import XCircle from '@lucide/svelte/icons/x-circle';
import KeyboardMusic from '@lucide/svelte/icons/keyboard-music';
import CircleX from '@lucide/svelte/icons/circle-x';
import ChartNoAxesColumn from '@lucide/svelte/icons/chart-no-axes-column';
import Folder from '@lucide/svelte/icons/folder';
import ArrowDownWideNarrow from '@lucide/svelte/icons/arrow-down-wide-narrow';
import ArrowUpNarrowWide from '@lucide/svelte/icons/arrow-up-narrow-wide';
import Grid2x2Icon from '@lucide/svelte/icons/grid-2x2';
import DiscAlbum from '@lucide/svelte/icons/disc-album';
import ListMusic from '@lucide/svelte/icons/list-music';
import Plus from '@lucide/svelte/icons/plus';
import Check from '@lucide/svelte/icons/check';
import X from '@lucide/svelte/icons/x';
import Image from '@lucide/svelte/icons/image';
import AlignJustify from '@lucide/svelte/icons/align-justify';

const iconRegistryLucide = {
	[IconType.Unknown]: HelpCircle,
	[IconType.Play]: Play,
	[IconType.Pause]: Pause,
	[IconType.Previous]: SkipBack,
	[IconType.Next]: SkipForward,
	[IconType.Playing]: Music2,
	[IconType.Note]: Music2,
	[IconType.PlayBack]: Undo2,
	[IconType.Back]: Undo2,
	[IconType.AlbumBack]: Undo2,
	[IconType.Speaker]: Volume2,
	[IconType.Mute]: VolumeX,
	[IconType.Remove]: XCircle,
	[IconType.Trash]: Trash2,
	[IconType.TrashRed]: Trash2,
	[IconType.Settings]: Settings,
	[IconType.Search]: Search,
	[IconType.QueueMusic]: ListPlus,
	[IconType.CleanQueue]: BrushCleaning,
	[IconType.RepeatNone]: Repeat,
	[IconType.RepeatPlayNone]: Repeat,
	[IconType.Repeat]: Repeat,
	[IconType.RepeatOne]: Repeat1,
	[IconType.Shuffle]: Shuffle,
	[IconType.Fullscreen]: Expand,
	[IconType.DragOn]: GripVertical,
	[IconType.DragOff]: Lock,
	[IconType.SaveLog]: FileText,
	[IconType.Equalizer]: KeyboardMusic,
	[IconType.Close]: CircleX,
	[IconType.Visualizer]: ChartNoAxesColumn,
	[IconType.Folder]: Folder,
	[IconType.SortAsc]: ArrowUpNarrowWide,
	[IconType.SortDesc]: ArrowDownWideNarrow,
	[IconType.MusicListTypeAll]: Grid2x2Icon,
	[IconType.MusicListTypeAlbum]: DiscAlbum,
	[IconType.MusicListTypeMusic]: Music2,
	[IconType.MusicListTypeFolder]: Folder,
	[IconType.MusicListTypePlaylist]: ListMusic,
	[IconType.PlaylistAdd]: Plus,
	[IconType.Check]: Check,
	[IconType.Cancel]: X,
	[IconType.Image]: Image,
	[IconType.Menu]: AlignJustify,
	[IconType.Queue]: ListMusic
};

export default iconRegistryLucide;
