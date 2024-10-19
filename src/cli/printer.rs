use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use itertools::Itertools;
use termal::{printmc, printmcln};

use crate::{
    core::{library::Song, messenger::Info},
    ext::duration_to_string,
};

pub fn now_playing(info: &Info, color: bool) {
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

    /*
                                  Out of Sight
                        by Smash Mouth from Smash Mouth

     01:07                      <||    |>    ||>                      02:56
    [==========================#-------------------------------------------]

                                     1/3179
                                      <6>
     */

    printmcln!(
        color,
        "
{'bold y}{title: ^80}{'_}
{: >by_from_off$}{'gr}by {'dc}{artist} {'gr}from {'dm}{album}

     {'w}{cur: <27}{'_ bold}<||{'y}{state: ^10}{'_fg}||>{'_ w}{total: >27}{'_}
    {'bold}[{'_ y}{before}{'w bold}{thumb}{'_ gr}{after}{'_ bold}]{'_}

{'gr}{playlist: ^80}
{disc_track: ^80}{'_}",
        ' '
    )
}

pub fn playlist(info: &Info, color: bool) {
    /*
                             ----<< playlist >>----

  1. Coldplay - Champion Of The World
  2. Smash Mouth - Out of Sight
  3. Alle Farben - Only Thing We Know
  4. X Ambassadors - Eye Of The Storm
  5. Imagine Dragons - Shots
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

pub fn song_list(songs: Vec<Song>, color: bool, send_time: Instant) {
    printmcln!(
        color,
        "{'bold y}{:<30} {'c}{:<20} {'m}{:28}{'_}",
        "Title",
        "Artist",
        "Album"
    );
    let mut total_dur = Duration::ZERO;
    for s in &songs {
        total_dur += s.length();
        print_song(s, color);
    }

    let elapsed = Instant::now() - send_time;

    printmcln!(
        color,
        "{'gr}{} songs ({}) in {:.4} s",
        songs.len(),
        duration_to_string(total_dur, true),
        elapsed.as_secs_f32(),
    );
}

fn print_song(s: &Song, color: bool) {
    printmc!(color, "{'dy}");
    print_elipsised(s.title(), 30);
    printmc!(color, " {'dc}");
    print_elipsised(s.artist(), 20);
    printmc!(color, " {'dm}");
    print_elipsised(s.album(), 28);
    printmcln!(color, "{'_}");
}

fn print_elipsised(s: &str, len: usize) {
    let mut ind = s.char_indices();
    let Some((i, _)) = ind.nth(len - 3) else {
        print!("{s:<len$}");
        return;
    };

    if ind.nth(3).is_some() {
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
        "  {'gr}{idx} {'dc}{} {'gr}- {'dy}{}{'_}",
        song.artist(),
        song.title(),
    );
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
        if self.is_playing {
            "||"
        } else {
            "|>"
        }
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

        if n == len {
            ""
        } else {
            "#"
        }
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
}