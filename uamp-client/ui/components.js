import Menu from "./components/menu.js";
import PlayerBar from "./components/player-bar.js";
import ScreenHeader from "./components/screen-header.js";
import SearchBar from "./components/search-bar.js";
import SvgIcon from "./components/svg-icon.js";

import AlbumScreen from "./screens/album.js";
import AlbumsScreen from "./screens/albums.js";
import ArtistScreen from "./screens/artist.js";
import ArtistsScreen from "./screens/artists.js";
import LibraryScreen from "./screens/library.js";
import PlaylistScreen from "./screens/playlist.js";

/** HTML components */
customElements.define("nav-menu", Menu);
customElements.define("player-bar", PlayerBar);
customElements.define("screen-header", ScreenHeader);
customElements.define("search-bar", SearchBar);
customElements.define("svg-icon", SvgIcon);

/** HTML Screen components */
customElements.define("library-screen", LibraryScreen);
customElements.define("albums-screen", AlbumsScreen);
customElements.define("album-screen", AlbumScreen);
customElements.define("artists-screen", ArtistsScreen);
customElements.define("artist-screen", ArtistScreen);
customElements.define("playlist-screen", PlaylistScreen);
