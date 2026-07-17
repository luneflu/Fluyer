export interface MusicData {
	id: number;
	path: string;
	filename: string;
	duration: number;

	title?: string;
	artist?: string;
	album?: string;
	albumArtist?: string;
	trackNumber?: string;
	image?: string;
	bitsPerSample?: number;
	sampleRate?: number;
	genre?: string;
	date?: string;
}

export interface AlbumData {
	name: string;
	artist: string;
	year: string;
	duration: string;
}

export interface FolderData {
	path: string;
}

export interface MusicPlayerSync {
	index: number;
	currentPosition: number;
	isPlaying: boolean;
	duration: number;
	repeatMode: RepeatMode;
}

export enum RepeatMode {
	None = 'repeatNone',
	One = 'repeatOne',
	All = 'repeat'
}

export enum MusicListType {
	All = 'all',
	Folder = 'folder',
	Album = 'album',
	Music = 'music',
	Playlist = 'playlist'
}

export interface PlaylistData {
	id?: number;
	name: string;
	image?: string;
	title?: string;
	artist?: string;
	paths: string[];
}
