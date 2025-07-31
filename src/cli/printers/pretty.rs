use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use image::imageops::FilterType;
use itertools::Itertools;
use termal::{
    codes, formatmc, printmc, printmcln,
    raw::{request, term_size},
};

use crate::{
    cli::Props,
    core::{
        config::{CacheSize, Config},
        library::Song,
        server::Info,
    },
    ext::duration_to_string,
};

pub fn info(info: &Info, conf: &Config, color: bool, lmc: bool) {
    now_playing(info, conf, color, lmc);

    if !info.before.is_empty() || !info.after.is_empty() {
        playlist(info, color);
    }

    playlist_config(info, color);
    footer(info, color);
}

pub fn now_playing(info: &Info, conf: &Config, color: bool, lmc: bool) {
    let title = info.title();

    let artist = info.artist();
    let album = info.album();

    let cur = info.cur_time();
    let state = info.state_btn();
    let total = info.total_time();

    let before = info.seek_before(70);
    let thumb = info.seek_thumb(70);
    let after = info.seek_after(70);

    let playlist = info.pos();
    let disc_track = info.disc_track();

    let by_from_off = 80_usize
        .saturating_sub(artist.chars().count() + album.chars().count() + 9)
        / 2;

    let vtop = info.vtop();
    let vbot = info.vbot();
    let volume = info.volume(color);
    let img = if color && conf.client_image_lookup() {
        info.image(conf)
    } else {
        String::new()
    };

    let clear = if lmc {
        codes::ERASE_ALL.to_string()
            + codes::ERASE_SCREEN
            + codes::move_to!(0, 0)
    } else {
        String::new()
    };

    /*
                                  Out of Sight
                        by Smash Mouth from Smash Mouth

     01:07                      <||    |>    ||>                      02:56
    [==========================#-------------------------------------------]

                                     1/3179                             ▃
                                      <6>                        v:  80 █
     */

    printmcln!(
        color,
        "{clear}{img}
{'bold y}{title: ^80}{'_}
{n: >by_from_off$}{'gr}by {'dc}{artist} {'gr}from {'dm}{album}

     {'w}{cur: <27}{'_ bold}<||{'y}{state: ^10}{'_fg}||>{'_ w}{total: >27}{'_}
    {'bold}[{'_ y}{before}{'w bold}{thumb}{'_ gr}{after}{'_ bold}]{'_}
{n: >72}{'u} {'_}
               {'gr}{playlist: ^50}       {'_}{vtop}{'_}
               {'gr}{disc_track: ^50}{volume} {'_ u}{vbot}{'_}",
        n = ""
    )
}

pub fn playlist(info: &Info, color: bool) {
    /*
                               ----<< playlist >>----

    1. Coldplay - Champion Of The World [3:25]
    2. Smash Mouth - Out of Sight [2:07]
    3. Alle Farben - Only Thing We Know [4:59]
    4. X Ambassadors - Eye Of The Storm [0:54]
    5. Imagine Dragons - Shots [9:56]
                                        ...
       */

    printmcln!(color, "\n{'gr}{: ^80}{'_}", "----<< playlist >>----");

    if info.playlist_top_continues() {
        printmcln!(color, "{'gr}{: ^80}", "...");
    } else {
        println!();
    }

    print_playlist(&info.before, info.before_start(), color);

    let idx: Cow<str> = info
        .playlist_pos
        .map_or("".into(), |i| format!("{}.", i + 1).into());
    printmcln!(
        color,
        "  {idx} {'c}{} {'_}- {'y}{}{'_}",
        info.artist(),
        info.title(),
    );

    print_playlist(&info.after, info.after_start(), color);

    if info.playlist_bot_continues() {
        printmcln!(color, "{'gr}{: ^80}", "...");
    } else {
        println!();
    }
}

pub fn playlist_config(info: &Info, color: bool) {
    /*
                                  4/13 > -/0
                         end: reset-playlist | add: m
    */

    let playlist_stack = info.playlist_stack();
    let config = format!(
        "end: {} | add: {}",
        info.playlist_end(),
        info.playlist_add_policy
    );

    printmcln!(
        color,
        "
{'gr}{playlist_stack: ^80}
{config: ^80}{'_}"
    )
}

pub fn footer(info: &Info, color: bool) {
    /*
    uamp                                                                      v0.5.4
         */
    printmcln!(color, "{'gr}uamp{: >76}{'_}", info.version());
}

pub fn song_list(songs: &[Song], props: &Props, send_time: Instant) {
    if props.verbosity >= 1 {
        verbose_song_list(songs, props.color, send_time);
    } else {
        compact_song_list(songs, props.color, send_time);
    }
}

pub fn compact_song_list(songs: &[Song], color: bool, send_time: Instant) {
    printmcln!(
        color,
        "{'bold u y}{:<30} {'c}{:<20} {'m}{:28}{'_}",
        "TITLE",
        "ARTIST",
        "ALBUM"
    );
    let mut total_dur = Duration::ZERO;
    for s in &songs[..songs.len().saturating_sub(1)] {
        total_dur += s.length();
        print_song(s, color);
    }

    if let Some(s) = songs.last() {
        total_dur += s.length();
        printmc!(color, "{'u uc8}");
        print_song(s, color);
    }

    let elapsed = Instant::now() - send_time;

    printmcln!(
        color,
        "{'gr ol}{} songs ({}) in {:.4} s{'_}",
        songs.len(),
        duration_to_string(total_dur, true),
        elapsed.as_secs_f32(),
    );
}

pub fn verbose_song_list(songs: &[Song], color: bool, send_time: Instant) {
    printmcln!(
        color,
        "{'bold u y}{:<45} {'c}{:<35} {'m}{:<7} {'g}{:<10}{'_}",
        "TITLE / ARTIST",
        "ALBUM / YEAR",
        "T / D",
        "LEN / GEN"
    );

    let mut total_dur = Duration::ZERO;
    for s in songs {
        total_dur += s.length();
        verbose_print_song(s, color);
    }

    let elapsed = Instant::now() - send_time;

    printmcln!(
        color,
        "{'gr}{} songs ({}) in {:.4} s {'_}",
        songs.len(),
        duration_to_string(total_dur, true),
        elapsed.as_secs_f32(),
    );
}

fn print_song(s: &Song, color: bool) {
    printmc!(color, "{'y}");
    print_elipsised(s.title(), 30);
    printmc!(color, " {'c}");
    print_elipsised(s.artist(), 20);
    printmc!(color, " {'m}");
    print_elipsised(s.album(), 28);
    printmcln!(color, "{'_}");
}

fn verbose_print_song(s: &Song, color: bool) {
    printmc!(color, "{'y}");
    print_elipsised(s.title(), 45);
    printmc!(color, " {'c}");
    print_elipsised(s.album(), 35);
    printmc!(color, " {'m}");
    print_elipsised(&s.track_str(), 7);
    printmc!(color, " {'g}");
    print_elipsised(&duration_to_string(s.length(), true), 10);
    printmcln!(color, "{'_}");

    printmc!(color, "{'u uc8 dy} ");
    print_elipsised(s.artist(), 44);
    printmc!(color, " {'dc} ");
    print_elipsised(&s.year_str(), 34);
    printmc!(color, " {'dm} ");
    print_elipsised(&s.disc_str(), 6);
    printmc!(color, " {'dg} ");
    print_elipsised(s.genre(), 9);
    printmcln!(color, "{'_}");
}

fn print_elipsised(s: &str, len: usize) {
    let mut ind = s.char_indices();
    let Some((i, _)) = ind.nth(len - 3) else {
        print!("{s:<len$}");
        return;
    };

    if ind.nth(2).is_some() {
        print!("{}...", &s[..i]);
    } else {
        print!("{s:<len$}");
    }
}

fn print_playlist<'a>(
    songs: impl IntoIterator<Item = &'a Song>,
    start_idx: Option<usize>,
    color: bool,
) {
    for (i, s) in songs.into_iter().enumerate() {
        print_playlist_song(s, start_idx.map(|idx| idx + i), color);
    }
}

fn print_playlist_song(song: &Song, idx: Option<usize>, color: bool) {
    let idx: Cow<str> = idx.map_or("".into(), |i| format!("{i}.").into());
    printmcln!(
        color,
        "  {'gr}{idx} {'dc}{} {'gr}- {'dy}{} {'bl}[{}]{'_}",
        song.artist(),
        song.title(),
        duration_to_string(song.length(), true),
    );
}

fn vblock(n: usize) -> char {
    [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'][n]
}

impl Info {
    fn title(&self) -> &str {
        self.now_playing_str(Song::title)
    }

    fn artist(&self) -> &str {
        self.now_playing_str(Song::artist)
    }

    fn album(&self) -> &str {
        self.now_playing_str(Song::album)
    }

    fn state_btn(&self) -> &'static str {
        if self.is_playing { "||" } else { "|>" }
    }

    fn cur_time(&self) -> Cow<'static, str> {
        Self::duration(self.timestamp.map(|t| t.current))
    }

    fn total_time(&self) -> Cow<'static, str> {
        Self::duration(self.timestamp.map(|t| t.total))
    }

    fn seek_before(&self, len: usize) -> Cow<'static, str> {
        let Some(n) = self.seek_pos(len) else {
            return "".into();
        };

        if n == 0 {
            "".into()
        } else {
            format!("{:=>n$}", '=').into()
        }
    }

    fn seek_thumb(&self, len: usize) -> &'static str {
        let Some(n) = self.seek_pos(len) else {
            return "";
        };

        if n == len { "" } else { "#" }
    }

    fn seek_after(&self, len: usize) -> Cow<'static, str> {
        let Some(n) = self.seek_pos(len) else {
            return format!("{:->len$}", '-').into();
        };

        let m = (len - n).saturating_sub(1);
        if m < 1 {
            "".into()
        } else {
            format!("{:->m$}", '-').into()
        }
    }

    fn disc_track(&self) -> Cow<'static, str> {
        let Some(s) = &self.now_playing else {
            return "<>".into();
        };

        let track: Cow<'static, str> =
            if s.track() == u32::MAX || s.track() == 0 {
                return "<>".into();
            } else {
                s.track().to_string().into()
            };

        if s.disc() == u32::MAX || s.disc() == 0 {
            format!("<{track}>").into()
        } else {
            format!("<{}-{track}>", s.disc()).into()
        }
    }

    fn version(&self) -> String {
        format!("v{}", self.version)
    }

    fn pos(&self) -> String {
        Self::playlist_pos(self.playlist_pos, self.playlist_len)
    }

    fn playlist_stack(&self) -> Cow<'static, str> {
        if self.playlist_stack.is_empty() {
            "-/-".into()
        } else {
            self.playlist_stack
                .iter()
                .rev()
                .map(|(pos, len)| Self::playlist_pos(*pos, *len))
                .join(" > ")
                .into()
        }
    }

    fn playlist_end(&self) -> Cow<'static, str> {
        self.playlist_end
            .as_ref()
            .map_or("--".into(), |a| a.to_string().into())
    }

    fn now_playing_str(&self, f: impl Fn(&Song) -> &str) -> &str {
        self.now_playing.as_ref().map_or("--", f)
    }

    fn playlist_top_continues(&self) -> bool {
        self.playlist_pos
            .map(|p| p - self.before.len() != 0)
            .unwrap_or(true)
    }

    fn playlist_bot_continues(&self) -> bool {
        self.playlist_pos
            .map(|p| p + self.after.len() + 1 < self.playlist_len)
            .unwrap_or(true)
    }

    fn before_start(&self) -> Option<usize> {
        self.playlist_pos
            .map(|i| (i + 1).saturating_sub(self.before.len()))
    }

    fn after_start(&self) -> Option<usize> {
        self.playlist_pos.map(|i| i + 2)
    }

    fn duration(dur: Option<Duration>) -> Cow<'static, str> {
        dur.map_or("--:--".into(), |d| duration_to_string(d, true).into())
    }

    fn seek_pos(&self, len: usize) -> Option<usize> {
        self.timestamp.map(|t| {
            (t.current.as_secs_f32() / t.total.as_secs_f32() * len as f32)
                as usize
        })
    }

    fn playlist_pos(cur: Option<usize>, len: usize) -> String {
        cur.map_or_else(|| format!("-/{len}"), |p| format!("{}/{len}", p + 1))
    }

    fn vtop(&self) -> char {
        vblock(((self.volume * 16.) as usize).saturating_sub(8))
    }

    fn vbot(&self) -> char {
        vblock(((self.volume * 16.) as usize).min(8))
    }

    fn volume(&self, color: bool) -> String {
        let vol = (self.volume * 100.).round() as u32;
        if self.mute {
            match vol {
                100.. => formatmc!(color, "m: {'strike}{vol}{'_strike}"),
                10.. => formatmc!(color, "m:  {'strike}{vol}{'_strike}"),
                _ => formatmc!(color, "m:   {'strike}{vol}{'_strike}"),
            }
        } else {
            format!("v: {vol: >3}")
        }
    }

    fn image(&self, conf: &Config) -> String {
        const IMG_CHAR_WIDTH: usize = 60;
        const DEFAULT_CHAR_RATIO: f32 = 0.5;

        // dbg!("Lookup");
        let mut res = String::new();
        let Some(s) = &self.now_playing else {
            return res;
        };
        let Ok(img) = s.lookup_cached_cover(conf, CacheSize::S256) else {
            return res;
        };

        let ratio = term_size()
            .map(|s| {
                if s.pixel_height == 0 || s.pixel_width == 0 {
                    DEFAULT_CHAR_RATIO
                } else {
                    (s.pixel_width as f32 * s.char_height as f32)
                        / (s.char_width as f32 * s.pixel_height as f32)
                }
            })
            .unwrap_or(DEFAULT_CHAR_RATIO);

        res.push('\n');

        let [w, h];
        if img.width() >= img.height() {
            w = IMG_CHAR_WIDTH;
            h = (img.height() as f32 * w as f32 / img.width() as f32 * ratio)
                as usize;
        } else {
            h = (IMG_CHAR_WIDTH as f32 * ratio) as usize;
            w = (img.width() as f32 * h as f32 / (img.height() as f32 * ratio))
                as usize;
        }

        // dbg!("Resize");
        let img = image::imageops::resize(
            &img,
            w as u32 * 2,
            h as u32 * 2,
            FilterType::Triangle,
        );

        let indent = (80 - w) / 2;
        let indent = format!("\n{:>indent$}", "");

        res += &indent[1..];

        let bg = request::default_bg_color(Duration::from_millis(100)).ok();

        if let Some(bg) = bg {
            termal::image::push_texel_quater_no_bg(
                &img,
                &mut res,
                &indent,
                Some(w),
                Some(h),
                bg.scale(),
            );
        } else {
            termal::image::push_texel_quater(
                &img,
                &mut res,
                &indent,
                Some(w),
                Some(h),
            );
        }

        res + codes::RESET + "\n"
    }
}
