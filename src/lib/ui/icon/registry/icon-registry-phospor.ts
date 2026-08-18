import { IconType } from '$lib/ui/icon/types';
import ArrowUUpLeftIcon from 'phosphor-svelte/lib/ArrowUUpLeftIcon';
import BroomIcon from 'phosphor-svelte/lib/BroomIcon';
import SortAscendingIcon from 'phosphor-svelte/lib/SortAscendingIcon';
import SortDescendingIcon from 'phosphor-svelte/lib/SortDescendingIcon';
import ChartBarIcon from 'phosphor-svelte/lib/ChartBarIcon';
import DotsSixVerticalIcon from 'phosphor-svelte/lib/DotsSixVerticalIcon';
import FadersIcon from 'phosphor-svelte/lib/FadersIcon';
import FileTextIcon from 'phosphor-svelte/lib/FileTextIcon';
import FolderIcon from 'phosphor-svelte/lib/FolderIcon';
import FrameCornersIcon from 'phosphor-svelte/lib/FrameCornersIcon';
import GearIcon from 'phosphor-svelte/lib/GearIcon';
import GridFourIcon from 'phosphor-svelte/lib/GridFourIcon';
import LockSimpleIcon from 'phosphor-svelte/lib/LockSimpleIcon';
import MagnifyingGlassIcon from 'phosphor-svelte/lib/MagnifyingGlassIcon';
import MusicNoteIcon from 'phosphor-svelte/lib/MusicNoteIcon';
import MusicNotesIcon from 'phosphor-svelte/lib/MusicNotesIcon';
import PauseCircleIcon from 'phosphor-svelte/lib/PauseCircleIcon';
import PlayCircleIcon from 'phosphor-svelte/lib/PlayCircleIcon';
import QuestionMarkIcon from 'phosphor-svelte/lib/QuestionMarkIcon';
import QueueIcon from 'phosphor-svelte/lib/QueueIcon';
import RepeatIcon from 'phosphor-svelte/lib/RepeatIcon';
import RepeatOnceIcon from 'phosphor-svelte/lib/RepeatOnceIcon';
import ShuffleIcon from 'phosphor-svelte/lib/ShuffleIcon';
import SkipBackCircleIcon from 'phosphor-svelte/lib/SkipBackCircleIcon';
import SkipForwardCircleIcon from 'phosphor-svelte/lib/SkipForwardCircleIcon';
import SpeakerHighIcon from 'phosphor-svelte/lib/SpeakerHighIcon';
import SpeakerXIcon from 'phosphor-svelte/lib/SpeakerXIcon';
import TrashIcon from 'phosphor-svelte/lib/TrashIcon';
import VinylRecordIcon from 'phosphor-svelte/lib/VinylRecordIcon';
import XCircleIcon from 'phosphor-svelte/lib/XCircleIcon';
import PlaylistIcon from 'phosphor-svelte/lib/PlaylistIcon';
import PlusIcon from 'phosphor-svelte/lib/PlusIcon';
import CheckIcon from 'phosphor-svelte/lib/CheckIcon';
import XIcon from 'phosphor-svelte/lib/XIcon';
import ImageIcon from 'phosphor-svelte/lib/ImageIcon';
import ListIcon from 'phosphor-svelte/lib/ListIcon';

const iconRegistryPhospor = {
	[IconType.Unknown]: QuestionMarkIcon,
	[IconType.Play]: PlayCircleIcon,
	[IconType.Pause]: PauseCircleIcon,
	[IconType.Previous]: SkipBackCircleIcon,
	[IconType.Next]: SkipForwardCircleIcon,
	[IconType.Playing]: MusicNotesIcon,
	[IconType.Note]: MusicNoteIcon,
	[IconType.PlayBack]: ArrowUUpLeftIcon,
	[IconType.Back]: ArrowUUpLeftIcon,
	[IconType.AlbumBack]: ArrowUUpLeftIcon,
	[IconType.Speaker]: SpeakerHighIcon,
	[IconType.Mute]: SpeakerXIcon,
	[IconType.Remove]: XCircleIcon,
	[IconType.Trash]: TrashIcon,
	[IconType.TrashRed]: TrashIcon,
	[IconType.Settings]: GearIcon,
	[IconType.Search]: MagnifyingGlassIcon,
	[IconType.QueueMusic]: QueueIcon,
	[IconType.CleanQueue]: BroomIcon,
	[IconType.RepeatNone]: RepeatIcon,
	[IconType.RepeatPlayNone]: RepeatIcon,
	[IconType.Repeat]: RepeatIcon,
	[IconType.RepeatOne]: RepeatOnceIcon,
	[IconType.Shuffle]: ShuffleIcon,
	[IconType.Fullscreen]: FrameCornersIcon,
	[IconType.DragOn]: DotsSixVerticalIcon,
	[IconType.DragOff]: LockSimpleIcon,
	[IconType.SaveLog]: FileTextIcon,
	[IconType.Equalizer]: FadersIcon,
	[IconType.Close]: XCircleIcon,
	[IconType.Visualizer]: ChartBarIcon,
	[IconType.Folder]: FolderIcon,
	[IconType.SortAsc]: SortAscendingIcon,
	[IconType.SortDesc]: SortDescendingIcon,
	[IconType.MusicListTypeAll]: GridFourIcon,
	[IconType.MusicListTypeAlbum]: VinylRecordIcon,
	[IconType.MusicListTypeMusic]: MusicNoteIcon,
	[IconType.MusicListTypeFolder]: FolderIcon,
	[IconType.MusicListTypePlaylist]: PlaylistIcon,
	[IconType.PlaylistAdd]: PlusIcon,
	[IconType.Check]: CheckIcon,
	[IconType.Cancel]: XIcon,
	[IconType.Image]: ImageIcon,
	[IconType.Menu]: ListIcon,
	[IconType.Queue]: QueueIcon
};

export default iconRegistryPhospor;
