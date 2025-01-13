use std::io;

use crate::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MediaKeyCode,
    ModifierKeyCode, MouseButton, MouseEvent, MouseEventKind,
};

// This is a lightly modified version of crossterm's ansi escape sequence parser: https://github.com/crossterm-rs/crossterm/blob/master/src/event/sys/unix/parse.rs

fn could_not_parse_event_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "Could not parse an event.")
}

impl Event {
    pub fn to_escape_sequence(&self) -> Vec<u8> {
        match self {
            Event::FocusGained => b"\x1B[I".to_vec(),
            Event::FocusLost => b"\x1B[O".to_vec(),
            Event::Key(key_event) => {
                let mut result = {
                    match key_event.code {
                        KeyCode::Backspace => b"\x7F".to_vec(),
                        KeyCode::Enter => b"\r".to_vec(),
                        KeyCode::Left => b"\x1B[D".to_vec(),
                        KeyCode::Right => b"\x1B[C".to_vec(),
                        KeyCode::Up => b"\x1B[A".to_vec(),
                        KeyCode::Down => b"\x1B[B".to_vec(),
                        KeyCode::Home => b"\x1B[H".to_vec(),
                        KeyCode::End => b"\x1B[F".to_vec(),
                        KeyCode::PageUp => b"\x1B[5~".to_vec(),
                        KeyCode::PageDown => b"\x1B[6~".to_vec(),
                        KeyCode::Tab => b"\t".to_vec(),
                        KeyCode::BackTab => b"\x1B[Z".to_vec(),
                        KeyCode::Delete => b"\x1B[3~".to_vec(),
                        KeyCode::Insert => b"\x1B[2~".to_vec(),
                        KeyCode::F(1) => b"\x1B[11~".to_vec(),
                        KeyCode::F(2) => b"\x1B[12~".to_vec(),
                        KeyCode::F(3) => b"\x1B[13~".to_vec(),
                        KeyCode::F(4) => b"\x1B[14~".to_vec(),
                        KeyCode::F(5) => b"\x1B[15~".to_vec(),
                        KeyCode::F(6) => b"\x1B[17~".to_vec(),
                        KeyCode::F(7) => b"\x1B[18~".to_vec(),
                        KeyCode::F(8) => b"\x1B[19~".to_vec(),
                        KeyCode::F(9) => b"\x1B[20~".to_vec(),
                        KeyCode::F(10) => b"\x1B[21~".to_vec(),
                        KeyCode::F(11) => b"\x1B[23~".to_vec(),
                        KeyCode::F(12) => b"\x1B[24~".to_vec(),
                        KeyCode::F(_) => todo!(),
                        KeyCode::Char(c) => {
                            let mut dst = vec![0; 1];
                            c.encode_utf8(&mut dst);
                            dst
                        }
                        KeyCode::Null => todo!(),
                        KeyCode::Esc => b"\x1B".to_vec(),
                        KeyCode::CapsLock => b"\x1B[57358u".to_vec(),
                        KeyCode::ScrollLock => b"\x1B[57359u".to_vec(),
                        KeyCode::NumLock => b"\x1B[57360u".to_vec(),
                        KeyCode::PrintScreen => b"\x1B[57361u".to_vec(),
                        KeyCode::Pause => b"\x1B[57362u".to_vec(),
                        KeyCode::Menu => b"\x1B[57363u".to_vec(),
                        KeyCode::KeypadBegin => todo!(),
                        KeyCode::Media(_) => todo!(),
                        KeyCode::Modifier(_) => todo!(),
                    }
                };

                if !key_event.modifiers.is_empty() {
                    match key_event.code {
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                            let last = result.pop().unwrap();
                            result.append(&mut b"1;1".to_vec());
                            result.push(last);
                        }
                        _ => {}
                    }
                }
                if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
                    match key_event.code {
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                            result[4] += 1;
                        }
                        _ => {}
                    }
                }
                if key_event.modifiers.intersects(KeyModifiers::ALT) {
                    match key_event.code {
                        KeyCode::Char(_) | KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
                            let mut prefix = b"\x1B".to_vec();
                            prefix.append(&mut result);
                            result = prefix;
                        }
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                            result[4] += 2;
                        }
                        _ => {}
                    }
                }
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            *result.last_mut().unwrap() = (c as u8) + 0x1 - b'a';
                        }
                        KeyCode::Backspace => {
                            *result.last_mut().unwrap() = b'\x08';
                        }
                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                            result[4] += 4;
                        }
                        _ => {}
                    }
                }
                result
            }
            Event::Mouse(_) => todo!(),
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => todo!(),
        }
    }
}

pub fn parse_event(buffer: &[u8]) -> io::Result<Option<Event>> {
    if buffer.is_empty() {
        return Ok(None);
    }

    match buffer[0] {
        b'\x1B' => {
            if buffer.len() == 1 {
                Ok(Some(Event::Key(KeyCode::Esc.into())))
            } else {
                match buffer[1] {
                    b'O' => {
                        if buffer.len() == 2 {
                            Ok(None)
                        } else {
                            match buffer[2] {
                                b'D' => Ok(Some(Event::Key(KeyCode::Left.into()))),
                                b'C' => Ok(Some(Event::Key(KeyCode::Right.into()))),
                                b'A' => Ok(Some(Event::Key(KeyCode::Up.into()))),
                                b'B' => Ok(Some(Event::Key(KeyCode::Down.into()))),
                                b'H' => Ok(Some(Event::Key(KeyCode::Home.into()))),
                                b'F' => Ok(Some(Event::Key(KeyCode::End.into()))),
                                // F1-F4
                                val @ b'P'..=b'S' => {
                                    Ok(Some(Event::Key(KeyCode::F(1 + val - b'P').into())))
                                }
                                _ => Err(could_not_parse_event_error()),
                            }
                        }
                    }
                    b'[' => parse_csi(buffer),
                    b'\x1B' => Ok(Some(Event::Key(KeyEvent::new(
                        KeyCode::Esc,
                        KeyModifiers::ALT,
                    )))),
                    _ => parse_event(&buffer[1..]).map(|event_option| {
                        event_option.map(|event| {
                            if let Event::Key(key_event) = event {
                                let mut alt_key_event = key_event;
                                alt_key_event.modifiers |= KeyModifiers::ALT;
                                Event::Key(alt_key_event)
                            } else {
                                event
                            }
                        })
                    }),
                }
            }
        }
        b'\r' => Ok(Some(Event::Key(KeyCode::Enter.into()))),
        // Issue #371: \n = 0xA, which is also the keycode for Ctrl+J. The only reason we get
        // newlines as input is because the terminal converts \r into \n for us. When we
        // enter raw mode, we disable that, so \n no longer has any meaning - it's better to
        // use Ctrl+J. Waiting to handle it here means it gets picked up later
        // b'\n' if !crate::terminal::sys::is_raw_mode_enabled() => {
        //     Ok(Some(Event::Key(KeyCode::Enter.into())))
        // }
        b'\t' => Ok(Some(Event::Key(KeyCode::Tab.into()))),
        b'\x7F' => Ok(Some(Event::Key(KeyCode::Backspace.into()))),
        c @ b'\x01'..=b'\x1A' => Ok(Some(Event::Key(KeyEvent::new(
            KeyCode::Char((c - 0x1 + b'a') as char),
            KeyModifiers::CONTROL,
        )))),
        c @ b'\x1C'..=b'\x1F' => Ok(Some(Event::Key(KeyEvent::new(
            KeyCode::Char((c - 0x1C + b'4') as char),
            KeyModifiers::CONTROL,
        )))),
        b'\0' => Ok(Some(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::CONTROL,
        )))),
        _ => parse_utf8_char(buffer).map(|maybe_char| {
            maybe_char
                .map(KeyCode::Char)
                .map(char_code_to_event)
                .map(Event::Key)
        }),
    }
}

// converts KeyCode to KeyEvent (adds shift modifier in case of uppercase characters)
fn char_code_to_event(code: KeyCode) -> KeyEvent {
    let modifiers = match code {
        KeyCode::Char(c) if c.is_uppercase() => KeyModifiers::SHIFT,
        _ => KeyModifiers::empty(),
    };
    KeyEvent::new(code, modifiers)
}

pub(crate) fn parse_csi(buffer: &[u8]) -> io::Result<Option<Event>> {
    assert!(buffer.starts_with(b"\x1B[")); // ESC [

    if buffer.len() == 2 {
        return Ok(None);
    }

    let input_event = match buffer[2] {
        b'[' => {
            if buffer.len() == 3 {
                None
            } else {
                match buffer[3] {
                    // NOTE (@imdaveho): cannot find when this occurs;
                    // having another '[' after ESC[ not a likely scenario
                    val @ b'A'..=b'E' => Some(Event::Key(KeyCode::F(1 + val - b'A').into())),
                    _ => return Err(could_not_parse_event_error()),
                }
            }
        }
        b'D' => Some(Event::Key(KeyCode::Left.into())),
        b'C' => Some(Event::Key(KeyCode::Right.into())),
        b'A' => Some(Event::Key(KeyCode::Up.into())),
        b'B' => Some(Event::Key(KeyCode::Down.into())),
        b'H' => Some(Event::Key(KeyCode::Home.into())),
        b'F' => Some(Event::Key(KeyCode::End.into())),
        b'Z' => Some(Event::Key(KeyEvent::new_with_kind(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
            KeyEventKind::Press,
        ))),
        b'M' => return parse_csi_normal_mouse(buffer),
        b'<' => return parse_csi_sgr_mouse(buffer),
        b'I' => Some(Event::FocusGained),
        b'O' => Some(Event::FocusLost),
        b';' => return parse_csi_modifier_key_code(buffer),
        // P, Q, and S for compatibility with Kitty keyboard protocol,
        // as the 1 in 'CSI 1 P' etc. must be omitted if there are no
        // modifiers pressed:
        // https://sw.kovidgoyal.net/kitty/keyboard-protocol/#legacy-functional-keys
        b'P' => Some(Event::Key(KeyCode::F(1).into())),
        b'Q' => Some(Event::Key(KeyCode::F(2).into())),
        b'S' => Some(Event::Key(KeyCode::F(4).into())),
        b'?' => match buffer[buffer.len() - 1] {
            b'u' => return Ok(None), //return parse_csi_keyboard_enhancement_flags(buffer),
            b'c' => return Ok(None), //return parse_csi_primary_device_attributes(buffer),
            _ => None,
        },
        b'0'..=b'9' => {
            // Numbered escape code.
            if buffer.len() == 3 {
                None
            } else {
                // The final byte of a CSI sequence can be in the range 64-126, so
                // let's keep reading anything else.
                let last_byte = buffer[buffer.len() - 1];
                if !(64..=126).contains(&last_byte) {
                    None
                } else {
                    if buffer.starts_with(b"\x1B[200~") {
                        return parse_csi_bracketed_paste(buffer);
                    }
                    match last_byte {
                        b'M' => return parse_csi_rxvt_mouse(buffer),
                        b'~' => return parse_csi_special_key_code(buffer),
                        b'u' => return parse_csi_u_encoded_key_code(buffer),
                        b'R' => return Ok(None), //parse_csi_cursor_position(buffer),
                        _ => return parse_csi_modifier_key_code(buffer),
                    }
                }
            }
        }
        _ => return Err(could_not_parse_event_error()),
    };

    Ok(input_event)
}

pub(crate) fn next_parsed<T>(iter: &mut dyn Iterator<Item = &str>) -> io::Result<T>
where
    T: std::str::FromStr,
{
    iter.next()
        .ok_or_else(could_not_parse_event_error)?
        .parse::<T>()
        .map_err(|_| could_not_parse_event_error())
}

fn modifier_and_kind_parsed(iter: &mut dyn Iterator<Item = &str>) -> io::Result<(u8, u8)> {
    let mut sub_split = iter
        .next()
        .ok_or_else(could_not_parse_event_error)?
        .split(':');

    let modifier_mask = next_parsed::<u8>(&mut sub_split)?;

    if let Ok(kind_code) = next_parsed::<u8>(&mut sub_split) {
        Ok((modifier_mask, kind_code))
    } else {
        Ok((modifier_mask, 1))
    }
}

fn parse_modifiers(mask: u8) -> KeyModifiers {
    let modifier_mask = mask.saturating_sub(1);
    let mut modifiers = KeyModifiers::empty();
    if modifier_mask & 1 != 0 {
        modifiers |= KeyModifiers::SHIFT;
    }
    if modifier_mask & 2 != 0 {
        modifiers |= KeyModifiers::ALT;
    }
    if modifier_mask & 4 != 0 {
        modifiers |= KeyModifiers::CONTROL;
    }
    if modifier_mask & 8 != 0 {
        modifiers |= KeyModifiers::SUPER;
    }
    if modifier_mask & 16 != 0 {
        modifiers |= KeyModifiers::HYPER;
    }
    if modifier_mask & 32 != 0 {
        modifiers |= KeyModifiers::META;
    }
    modifiers
}

fn parse_modifiers_to_state(mask: u8) -> KeyEventState {
    let modifier_mask = mask.saturating_sub(1);
    let mut state = KeyEventState::empty();
    if modifier_mask & 64 != 0 {
        state |= KeyEventState::CAPS_LOCK;
    }
    if modifier_mask & 128 != 0 {
        state |= KeyEventState::NUM_LOCK;
    }
    state
}

fn parse_key_event_kind(kind: u8) -> KeyEventKind {
    match kind {
        1 => KeyEventKind::Press,
        2 => KeyEventKind::Repeat,
        3 => KeyEventKind::Release,
        _ => KeyEventKind::Press,
    }
}

pub(crate) fn parse_csi_modifier_key_code(buffer: &[u8]) -> io::Result<Option<Event>> {
    assert!(buffer.starts_with(b"\x1B[")); // ESC [
    //
    let s = std::str::from_utf8(&buffer[2..buffer.len() - 1])
        .map_err(|_| could_not_parse_event_error())?;
    let mut split = s.split(';');

    split.next();

    let (modifiers, kind) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
            )
        } else if buffer.len() > 3 {
            (
                parse_modifiers(
                    (buffer[buffer.len() - 2] as char)
                        .to_digit(10)
                        .ok_or_else(could_not_parse_event_error)? as u8,
                ),
                KeyEventKind::Press,
            )
        } else {
            (KeyModifiers::NONE, KeyEventKind::Press)
        };
    let key = buffer[buffer.len() - 1];

    let keycode = match key {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        b'F' => KeyCode::End,
        b'H' => KeyCode::Home,
        b'P' => KeyCode::F(1),
        b'Q' => KeyCode::F(2),
        b'R' => KeyCode::F(3),
        b'S' => KeyCode::F(4),
        _ => return Err(could_not_parse_event_error()),
    };

    let input_event = Event::Key(KeyEvent::new_with_kind(keycode, modifiers, kind));

    Ok(Some(input_event))
}

fn translate_functional_key_code(codepoint: u32) -> Option<(KeyCode, KeyEventState)> {
    if let Some(keycode) = match codepoint {
        57399 => Some(KeyCode::Char('0')),
        57400 => Some(KeyCode::Char('1')),
        57401 => Some(KeyCode::Char('2')),
        57402 => Some(KeyCode::Char('3')),
        57403 => Some(KeyCode::Char('4')),
        57404 => Some(KeyCode::Char('5')),
        57405 => Some(KeyCode::Char('6')),
        57406 => Some(KeyCode::Char('7')),
        57407 => Some(KeyCode::Char('8')),
        57408 => Some(KeyCode::Char('9')),
        57409 => Some(KeyCode::Char('.')),
        57410 => Some(KeyCode::Char('/')),
        57411 => Some(KeyCode::Char('*')),
        57412 => Some(KeyCode::Char('-')),
        57413 => Some(KeyCode::Char('+')),
        57414 => Some(KeyCode::Enter),
        57415 => Some(KeyCode::Char('=')),
        57416 => Some(KeyCode::Char(',')),
        57417 => Some(KeyCode::Left),
        57418 => Some(KeyCode::Right),
        57419 => Some(KeyCode::Up),
        57420 => Some(KeyCode::Down),
        57421 => Some(KeyCode::PageUp),
        57422 => Some(KeyCode::PageDown),
        57423 => Some(KeyCode::Home),
        57424 => Some(KeyCode::End),
        57425 => Some(KeyCode::Insert),
        57426 => Some(KeyCode::Delete),
        57427 => Some(KeyCode::KeypadBegin),
        _ => None,
    } {
        return Some((keycode, KeyEventState::KEYPAD));
    }

    if let Some(keycode) = match codepoint {
        57358 => Some(KeyCode::CapsLock),
        57359 => Some(KeyCode::ScrollLock),
        57360 => Some(KeyCode::NumLock),
        57361 => Some(KeyCode::PrintScreen),
        57362 => Some(KeyCode::Pause),
        57363 => Some(KeyCode::Menu),
        57376 => Some(KeyCode::F(13)),
        57377 => Some(KeyCode::F(14)),
        57378 => Some(KeyCode::F(15)),
        57379 => Some(KeyCode::F(16)),
        57380 => Some(KeyCode::F(17)),
        57381 => Some(KeyCode::F(18)),
        57382 => Some(KeyCode::F(19)),
        57383 => Some(KeyCode::F(20)),
        57384 => Some(KeyCode::F(21)),
        57385 => Some(KeyCode::F(22)),
        57386 => Some(KeyCode::F(23)),
        57387 => Some(KeyCode::F(24)),
        57388 => Some(KeyCode::F(25)),
        57389 => Some(KeyCode::F(26)),
        57390 => Some(KeyCode::F(27)),
        57391 => Some(KeyCode::F(28)),
        57392 => Some(KeyCode::F(29)),
        57393 => Some(KeyCode::F(30)),
        57394 => Some(KeyCode::F(31)),
        57395 => Some(KeyCode::F(32)),
        57396 => Some(KeyCode::F(33)),
        57397 => Some(KeyCode::F(34)),
        57398 => Some(KeyCode::F(35)),
        57428 => Some(KeyCode::Media(MediaKeyCode::Play)),
        57429 => Some(KeyCode::Media(MediaKeyCode::Pause)),
        57430 => Some(KeyCode::Media(MediaKeyCode::PlayPause)),
        57431 => Some(KeyCode::Media(MediaKeyCode::Reverse)),
        57432 => Some(KeyCode::Media(MediaKeyCode::Stop)),
        57433 => Some(KeyCode::Media(MediaKeyCode::FastForward)),
        57434 => Some(KeyCode::Media(MediaKeyCode::Rewind)),
        57435 => Some(KeyCode::Media(MediaKeyCode::TrackNext)),
        57436 => Some(KeyCode::Media(MediaKeyCode::TrackPrevious)),
        57437 => Some(KeyCode::Media(MediaKeyCode::Record)),
        57438 => Some(KeyCode::Media(MediaKeyCode::LowerVolume)),
        57439 => Some(KeyCode::Media(MediaKeyCode::RaiseVolume)),
        57440 => Some(KeyCode::Media(MediaKeyCode::MuteVolume)),
        57441 => Some(KeyCode::Modifier(ModifierKeyCode::Shift)),
        57442 => Some(KeyCode::Modifier(ModifierKeyCode::Control)),
        57443 => Some(KeyCode::Modifier(ModifierKeyCode::Alt)),
        57444 => Some(KeyCode::Modifier(ModifierKeyCode::Super)),
        57445 => Some(KeyCode::Modifier(ModifierKeyCode::Hyper)),
        57446 => Some(KeyCode::Modifier(ModifierKeyCode::Meta)),
        57447 => Some(KeyCode::Modifier(ModifierKeyCode::Shift)),
        57448 => Some(KeyCode::Modifier(ModifierKeyCode::Control)),
        57449 => Some(KeyCode::Modifier(ModifierKeyCode::Alt)),
        57450 => Some(KeyCode::Modifier(ModifierKeyCode::Super)),
        57451 => Some(KeyCode::Modifier(ModifierKeyCode::Hyper)),
        57452 => Some(KeyCode::Modifier(ModifierKeyCode::Meta)),
        57453 => Some(KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift)),
        57454 => Some(KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift)),
        _ => None,
    } {
        return Some((keycode, KeyEventState::empty()));
    }

    None
}

pub(crate) fn parse_csi_u_encoded_key_code(buffer: &[u8]) -> io::Result<Option<Event>> {
    assert!(buffer.starts_with(b"\x1B[")); // ESC [
    assert!(buffer.ends_with(b"u"));

    // This function parses `CSI … u` sequences. These are sequences defined in either
    // the `CSI u` (a.k.a. "Fix Keyboard Input on Terminals - Please", https://www.leonerd.org.uk/hacks/fixterms/)
    // or Kitty Keyboard Protocol (https://sw.kovidgoyal.net/kitty/keyboard-protocol/) specifications.
    // This CSI sequence is a tuple of semicolon-separated numbers.
    let s = std::str::from_utf8(&buffer[2..buffer.len() - 1])
        .map_err(|_| could_not_parse_event_error())?;
    let mut split = s.split(';');

    // In `CSI u`, this is parsed as:
    //
    //     CSI codepoint ; modifiers u
    //     codepoint: ASCII Dec value
    //
    // The Kitty Keyboard Protocol extends this with optional components that can be
    // enabled progressively. The full sequence is parsed as:
    //
    //     CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ; text-as-codepoints u
    let mut codepoints = split
        .next()
        .ok_or_else(could_not_parse_event_error)?
        .split(':');

    let codepoint = codepoints
        .next()
        .ok_or_else(could_not_parse_event_error)?
        .parse::<u32>()
        .map_err(|_| could_not_parse_event_error())?;

    let (mut modifiers, kind, state_from_modifiers) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
                parse_modifiers_to_state(modifier_mask),
            )
        } else {
            (KeyModifiers::NONE, KeyEventKind::Press, KeyEventState::NONE)
        };

    let (mut keycode, state_from_keycode) = {
        if let Some((special_key_code, state)) = translate_functional_key_code(codepoint) {
            (special_key_code, state)
        } else if let Some(c) = char::from_u32(codepoint) {
            (
                match c {
                    '\x1B' => KeyCode::Esc,
                    '\r' => KeyCode::Enter,
                    // Issue #371: \n = 0xA, which is also the keycode for Ctrl+J. The only reason
                    // we get newlines as input is because the terminal converts
                    // \r into \n for us. When we enter raw mode, we disable
                    // that, so \n no longer has any meaning - it's better to
                    // use Ctrl+J. Waiting to handle it here means it gets picked up later
                    // '\n' if !crate::terminal::sys::is_raw_mode_enabled() => KeyCode::Enter,
                    '\t' => {
                        if modifiers.contains(KeyModifiers::SHIFT) {
                            KeyCode::BackTab
                        } else {
                            KeyCode::Tab
                        }
                    }
                    '\x7F' => KeyCode::Backspace,
                    _ => KeyCode::Char(c),
                },
                KeyEventState::empty(),
            )
        } else {
            return Err(could_not_parse_event_error());
        }
    };

    if let KeyCode::Modifier(modifier_keycode) = keycode {
        match modifier_keycode {
            ModifierKeyCode::Alt => modifiers.set(KeyModifiers::ALT, true),
            ModifierKeyCode::Control => modifiers.set(KeyModifiers::CONTROL, true),
            ModifierKeyCode::Shift => modifiers.set(KeyModifiers::SHIFT, true),
            ModifierKeyCode::Super => modifiers.set(KeyModifiers::SUPER, true),
            ModifierKeyCode::Hyper => modifiers.set(KeyModifiers::HYPER, true),
            ModifierKeyCode::Meta => modifiers.set(KeyModifiers::META, true),
            _ => {}
        }
    }

    // When the "report alternate keys" flag is enabled in the Kitty Keyboard Protocol
    // and the terminal sends a keyboard event containing shift, the sequence will
    // contain an additional codepoint separated by a ':' character which contains
    // the shifted character according to the keyboard layout.
    if modifiers.contains(KeyModifiers::SHIFT) {
        if let Some(shifted_c) = codepoints
            .next()
            .and_then(|codepoint| codepoint.parse::<u32>().ok())
            .and_then(char::from_u32)
        {
            keycode = KeyCode::Char(shifted_c);
            modifiers.set(KeyModifiers::SHIFT, false);
        }
    }

    let input_event = Event::Key(KeyEvent::new_with_kind_and_state(
        keycode,
        modifiers,
        kind,
        state_from_keycode | state_from_modifiers,
    ));

    Ok(Some(input_event))
}

pub(crate) fn parse_csi_special_key_code(buffer: &[u8]) -> io::Result<Option<Event>> {
    assert!(buffer.starts_with(b"\x1B[")); // ESC [
    assert!(buffer.ends_with(b"~"));

    let s = std::str::from_utf8(&buffer[2..buffer.len() - 1])
        .map_err(|_| could_not_parse_event_error())?;
    let mut split = s.split(';');

    // This CSI sequence can be a list of semicolon-separated numbers.
    let first = next_parsed::<u8>(&mut split)?;

    let (modifiers, kind, state) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
                parse_modifiers_to_state(modifier_mask),
            )
        } else {
            (KeyModifiers::NONE, KeyEventKind::Press, KeyEventState::NONE)
        };

    let keycode = match first {
        1 | 7 => KeyCode::Home,
        2 => KeyCode::Insert,
        3 => KeyCode::Delete,
        4 | 8 => KeyCode::End,
        5 => KeyCode::PageUp,
        6 => KeyCode::PageDown,
        v @ 11..=15 => KeyCode::F(v - 10),
        v @ 17..=21 => KeyCode::F(v - 11),
        v @ 23..=26 => KeyCode::F(v - 12),
        v @ 28..=29 => KeyCode::F(v - 15),
        v @ 31..=34 => KeyCode::F(v - 17),
        _ => return Err(could_not_parse_event_error()),
    };

    let input_event = Event::Key(KeyEvent::new_with_kind_and_state(
        keycode, modifiers, kind, state,
    ));

    Ok(Some(input_event))
}

pub(crate) fn parse_csi_rxvt_mouse(buffer: &[u8]) -> io::Result<Option<Event>> {
    // rxvt mouse encoding:
    // ESC [ Cb ; Cx ; Cy ; M

    assert!(buffer.starts_with(b"\x1B[")); // ESC [
    assert!(buffer.ends_with(b"M"));

    let s = std::str::from_utf8(&buffer[2..buffer.len() - 1])
        .map_err(|_| could_not_parse_event_error())?;
    let mut split = s.split(';');

    let cb = next_parsed::<u8>(&mut split)?
        .checked_sub(32)
        .ok_or_else(could_not_parse_event_error)?;
    let (kind, modifiers) = parse_cb(cb)?;

    let cx = next_parsed::<u16>(&mut split)? - 1;
    let cy = next_parsed::<u16>(&mut split)? - 1;

    Ok(Some(Event::Mouse(MouseEvent {
        kind,
        column: cx,
        row: cy,
        modifiers,
    })))
}

pub(crate) fn parse_csi_normal_mouse(buffer: &[u8]) -> io::Result<Option<Event>> {
    // Normal mouse encoding: ESC [ M CB Cx Cy (6 characters only).

    assert!(buffer.starts_with(b"\x1B[M")); // ESC [ M

    if buffer.len() < 6 {
        return Ok(None);
    }

    let cb = buffer[3]
        .checked_sub(32)
        .ok_or_else(could_not_parse_event_error)?;
    let (kind, modifiers) = parse_cb(cb)?;

    // See http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking
    // The upper left character position on the terminal is denoted as 1,1.
    // Subtract 1 to keep it synced with cursor
    let cx = u16::from(buffer[4].saturating_sub(32)) - 1;
    let cy = u16::from(buffer[5].saturating_sub(32)) - 1;

    Ok(Some(Event::Mouse(MouseEvent {
        kind,
        column: cx,
        row: cy,
        modifiers,
    })))
}

pub(crate) fn parse_csi_sgr_mouse(buffer: &[u8]) -> io::Result<Option<Event>> {
    // ESC [ < Cb ; Cx ; Cy (;) (M or m)

    assert!(buffer.starts_with(b"\x1B[<")); // ESC [ <

    if !buffer.ends_with(b"m") && !buffer.ends_with(b"M") {
        return Ok(None);
    }

    let s = std::str::from_utf8(&buffer[3..buffer.len() - 1])
        .map_err(|_| could_not_parse_event_error())?;
    let mut split = s.split(';');

    let cb = next_parsed::<u8>(&mut split)?;
    let (kind, modifiers) = parse_cb(cb)?;

    // See http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking
    // The upper left character position on the terminal is denoted as 1,1.
    // Subtract 1 to keep it synced with cursor
    let cx = next_parsed::<u16>(&mut split)? - 1;
    let cy = next_parsed::<u16>(&mut split)? - 1;

    // When button 3 in Cb is used to represent mouse release, you can't tell which button was
    // released. SGR mode solves this by having the sequence end with a lowercase m if it's a
    // button release and an uppercase M if it's a button press.
    //
    // We've already checked that the last character is a lowercase or uppercase M at the start of
    // this function, so we just need one if.
    let kind = if buffer.last() == Some(&b'm') {
        match kind {
            MouseEventKind::Down(button) => MouseEventKind::Up(button),
            other => other,
        }
    } else {
        kind
    };

    Ok(Some(Event::Mouse(MouseEvent {
        kind,
        column: cx,
        row: cy,
        modifiers,
    })))
}

/// Cb is the byte of a mouse input that contains the button being used, the key modifiers being
/// held and whether the mouse is dragging or not.
///
/// Bit layout of cb, from low to high:
///
/// - button number
/// - button number
/// - shift
/// - meta (alt)
/// - control
/// - mouse is dragging
/// - button number
/// - button number
fn parse_cb(cb: u8) -> io::Result<(MouseEventKind, KeyModifiers)> {
    let button_number = (cb & 0b0000_0011) | ((cb & 0b1100_0000) >> 4);
    let dragging = cb & 0b0010_0000 == 0b0010_0000;

    let kind = match (button_number, dragging) {
        (0, false) => MouseEventKind::Down(MouseButton::Left),
        (1, false) => MouseEventKind::Down(MouseButton::Middle),
        (2, false) => MouseEventKind::Down(MouseButton::Right),
        (0, true) => MouseEventKind::Drag(MouseButton::Left),
        (1, true) => MouseEventKind::Drag(MouseButton::Middle),
        (2, true) => MouseEventKind::Drag(MouseButton::Right),
        (3, false) => MouseEventKind::Up(MouseButton::Left),
        (3, true) | (4, true) | (5, true) => MouseEventKind::Moved,
        (4, false) => MouseEventKind::ScrollUp,
        (5, false) => MouseEventKind::ScrollDown,
        (6, false) => MouseEventKind::ScrollLeft,
        (7, false) => MouseEventKind::ScrollRight,
        // We do not support other buttons.
        _ => return Err(could_not_parse_event_error()),
    };

    let mut modifiers = KeyModifiers::empty();

    if cb & 0b0000_0100 == 0b0000_0100 {
        modifiers |= KeyModifiers::SHIFT;
    }
    if cb & 0b0000_1000 == 0b0000_1000 {
        modifiers |= KeyModifiers::ALT;
    }
    if cb & 0b0001_0000 == 0b0001_0000 {
        modifiers |= KeyModifiers::CONTROL;
    }

    Ok((kind, modifiers))
}

pub(crate) fn parse_csi_bracketed_paste(buffer: &[u8]) -> io::Result<Option<Event>> {
    // ESC [ 2 0 0 ~ pasted text ESC 2 0 1 ~
    assert!(buffer.starts_with(b"\x1B[200~"));

    if !buffer.ends_with(b"\x1b[201~") {
        Ok(None)
    } else {
        let paste = String::from_utf8_lossy(&buffer[6..buffer.len() - 6]).to_string();
        Ok(Some(Event::Paste(paste)))
    }
}

pub(crate) fn parse_utf8_char(buffer: &[u8]) -> io::Result<Option<char>> {
    match std::str::from_utf8(buffer) {
        Ok(s) => {
            let ch = s.chars().next().ok_or_else(could_not_parse_event_error)?;

            Ok(Some(ch))
        }
        Err(_) => {
            // from_utf8 failed, but we have to check if we need more bytes for code point
            // and if all the bytes we have no are valid

            let required_bytes = match buffer[0] {
                // https://en.wikipedia.org/wiki/UTF-8#Description
                (0x00..=0x7F) => 1, // 0xxxxxxx
                (0xC0..=0xDF) => 2, // 110xxxxx 10xxxxxx
                (0xE0..=0xEF) => 3, // 1110xxxx 10xxxxxx 10xxxxxx
                (0xF0..=0xF7) => 4, // 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
                (0x80..=0xBF) | (0xF8..=0xFF) => return Err(could_not_parse_event_error()),
            };

            // More than 1 byte, check them for 10xxxxxx pattern
            if required_bytes > 1 && buffer.len() > 1 {
                for byte in &buffer[1..] {
                    if byte & !0b0011_1111 != 0b1000_0000 {
                        return Err(could_not_parse_event_error());
                    }
                }
            }

            if buffer.len() < required_bytes {
                // All bytes looks good so far, but we need more of them
                Ok(None)
            } else {
                Err(could_not_parse_event_error())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_esc_key() {
        assert_eq!(
            parse_event(b"\x1B").unwrap(),
            Some(Event::Key(KeyCode::Esc.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Esc.into()).to_escape_sequence(),
            b"\x1B"
        );

        assert_eq!(
            parse_event(b"\x1B\x1B").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B\x1B"
        );

        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::CONTROL)).to_escape_sequence(),
            b"\x1B"
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::ALT | KeyModifiers::CONTROL
            ))
            .to_escape_sequence(),
            b"\x1B\x1B"
        );
    }

    #[test]
    fn test_backspace() {
        assert_eq!(
            parse_event(b"\x7F").unwrap(),
            Some(Event::Key(KeyCode::Backspace.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Backspace.into()).to_escape_sequence(),
            b"\x7F"
        );

        assert_eq!(
            parse_event(b"\x1B\x7F").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::ALT
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B\x7F",
        );

        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL))
                .to_escape_sequence(),
            b"\x08",
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            ))
            .to_escape_sequence(),
            b"\x1B\x08",
        );
    }

    #[test]
    fn test_focus_gained() {
        assert_eq!(parse_event(b"\x1B[I").unwrap(), Some(Event::FocusGained));
        assert_eq!(Event::FocusGained.to_escape_sequence(), b"\x1B[I");
    }

    #[test]
    fn test_focus_lost() {
        assert_eq!(parse_event(b"\x1B[O").unwrap(), Some(Event::FocusLost));
        assert_eq!(Event::FocusLost.to_escape_sequence(), b"\x1B[O");
    }

    #[test]
    fn test_enter() {
        assert_eq!(
            parse_event(b"\r").unwrap(),
            Some(Event::Key(KeyCode::Enter.into()))
        );
        assert_eq!(
            Event::Key(KeyCode::Enter.into()).to_escape_sequence(),
            b"\r"
        );

        assert_eq!(
            parse_event(b"\x1B\r").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B\r"
        );
    }

    #[test]
    fn test_alt_key() {
        assert_eq!(
            parse_event(b"\x1Bc").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::ALT
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1Bc"
        );
    }

    #[test]
    fn test_ctrl_key() {
        assert_eq!(
            parse_event(b"\x03").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
                .to_escape_sequence(),
            b"\x03"
        );
    }

    #[test]
    fn test_alt_shift() {
        assert_eq!(
            parse_event(b"\x1BH").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('H'),
                KeyModifiers::ALT | KeyModifiers::SHIFT
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Char('H'),
                KeyModifiers::ALT | KeyModifiers::SHIFT
            ))
            .to_escape_sequence(),
            b"\x1BH"
        )
    }

    #[test]
    fn test_ctrl_alt() {
        assert_eq!(
            parse_event(b"\x1B\x14").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('t'),
                KeyModifiers::ALT | KeyModifiers::CONTROL
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Char('t'),
                KeyModifiers::ALT | KeyModifiers::CONTROL
            ))
            .to_escape_sequence(),
            b"\x1B\x14"
        );
    }

    #[test]
    fn test_left_arrow() {
        assert_eq!(
            parse_event(b"\x1B[D").unwrap(),
            Some(Event::Key(KeyCode::Left.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Left.into()).to_escape_sequence(),
            b"\x1B[D"
        );

        assert_eq!(
            parse_event(b"\x1B[2D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            parse_event(b"\x1B[1;2D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)).to_escape_sequence(),
            b"\x1B[1;2D"
        );

        assert_eq!(
            parse_event(b"\x1B[1;5D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)).to_escape_sequence(),
            b"\x1B[1;5D"
        );

        assert_eq!(
            parse_event(b"\x1B[1;3D").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B[1;3D"
        );

        assert_eq!(
            parse_event(b"\x1B[1;7D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            ))
            .to_escape_sequence(),
            b"\x1B[1;7D"
        );

        assert_eq!(
            parse_event(b"\x1B[1;8D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            ))
            .to_escape_sequence(),
            b"\x1B[1;8D"
        );
    }

    #[test]
    fn test_right_arrow() {
        assert_eq!(
            parse_event(b"\x1B[C").unwrap(),
            Some(Event::Key(KeyCode::Right.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Right.into()).to_escape_sequence(),
            b"\x1B[C"
        );

        assert_eq!(
            parse_event(b"\x1B[2C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            parse_event(b"\x1B[1;2C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)).to_escape_sequence(),
            b"\x1B[1;2C"
        );

        assert_eq!(
            parse_event(b"\x1B[1;5C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)).to_escape_sequence(),
            b"\x1B[1;5C"
        );

        assert_eq!(
            parse_event(b"\x1B[1;3C").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B[1;3C"
        );

        assert_eq!(
            parse_event(b"\x1B[1;7C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            ))
            .to_escape_sequence(),
            b"\x1B[1;7C"
        );

        assert_eq!(
            parse_event(b"\x1B[1;8C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            ))
            .to_escape_sequence(),
            b"\x1B[1;8C"
        );
    }

    #[test]
    fn test_up_arrow() {
        assert_eq!(
            parse_event(b"\x1B[A").unwrap(),
            Some(Event::Key(KeyCode::Up.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Up.into()).to_escape_sequence(),
            b"\x1B[A"
        );

        assert_eq!(
            parse_event(b"\x1B[2A").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)))
        );
        assert_eq!(
            parse_event(b"\x1B[1;2A").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)).to_escape_sequence(),
            b"\x1B[1;2A"
        );

        assert_eq!(
            parse_event(b"\x1B[1;5A").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::CONTROL
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)).to_escape_sequence(),
            b"\x1B[1;5A"
        );

        assert_eq!(
            parse_event(b"\x1B[1;3A").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B[1;3A"
        );

        assert_eq!(
            parse_event(b"\x1B[1;7A").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            ))
            .to_escape_sequence(),
            b"\x1B[1;7A"
        );

        assert_eq!(
            parse_event(b"\x1B[1;8A").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Up,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            ))
            .to_escape_sequence(),
            b"\x1B[1;8A"
        );
    }

    #[test]
    fn test_down_arrow() {
        assert_eq!(
            parse_event(b"\x1B[B").unwrap(),
            Some(Event::Key(KeyCode::Down.into())),
        );
        assert_eq!(
            Event::Key(KeyCode::Down.into()).to_escape_sequence(),
            b"\x1B[B"
        );

        assert_eq!(
            parse_event(b"\x1B[2B").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            parse_event(b"\x1B[1;2B").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)).to_escape_sequence(),
            b"\x1B[1;2B"
        );

        assert_eq!(
            parse_event(b"\x1B[1;5B").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::CONTROL
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL)).to_escape_sequence(),
            b"\x1B[1;5B"
        );

        assert_eq!(
            parse_event(b"\x1B[1;3B").unwrap(),
            Some(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::ALT)))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::ALT)).to_escape_sequence(),
            b"\x1B[1;3B"
        );

        assert_eq!(
            parse_event(b"\x1B[1;7B").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            ))
            .to_escape_sequence(),
            b"\x1B[1;7B"
        );

        assert_eq!(
            parse_event(b"\x1B[1;8B").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            )))
        );
        assert_eq!(
            Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            ))
            .to_escape_sequence(),
            b"\x1B[1;8B"
        );
    }

    #[test]
    fn test_parse_event_subsequent_calls() {
        // The main purpose of this test is to check if we're passing
        // correct slice to other parse_ functions.

        // parse_csi_cursor_position
        // assert_eq!(
        //     parse_event(b"\x1B[20;10R").unwrap(),
        //     Some(InternalEvent::CursorPosition(9, 19))
        // );

        // parse_csi

        assert_eq!(
            parse_event(b"\x1B[C").unwrap(),
            Some(Event::Key(KeyCode::Right.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Right.into()).to_escape_sequence(),
            b"\x1B[2C"
        );

        assert_eq!(
            parse_event(b"\x1B[A").unwrap(),
            Some(Event::Key(KeyCode::Up.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Up.into()).to_escape_sequence(),
            b"\x1B[2A"
        );

        assert_eq!(
            parse_event(b"\x1B[B").unwrap(),
            Some(Event::Key(KeyCode::Down.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Down.into()).to_escape_sequence(),
            b"\x1B[2B"
        );

        assert_eq!(
            parse_event(b"\x1B[H").unwrap(),
            Some(Event::Key(KeyCode::Home.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Home.into()).to_escape_sequence(),
            b"\x1B[2H"
        );

        assert_eq!(
            parse_event(b"\x1B[F").unwrap(),
            Some(Event::Key(KeyCode::End.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::End.into()).to_escape_sequence(),
            b"\x1B[2F"
        );

        // parse_csi_special_key_code
        assert_eq!(
            parse_event(b"\x1B[3~").unwrap(),
            Some(Event::Key(KeyCode::Delete.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Delete.into()).to_escape_sequence(),
            b"\x1B[3~"
        );

        assert_eq!(
            parse_event(b"\x1B[2~").unwrap(),
            Some(Event::Key(KeyCode::Insert.into())),
        );

        assert_eq!(
            Event::Key(KeyCode::Insert.into()).to_escape_sequence(),
            b"\x1B[2~"
        );

        // parse_csi_bracketed_paste
        assert_eq!(
            parse_event(b"\x1B[200~on and on and on\x1B[201~").unwrap(),
            Some(Event::Paste("on and on and on".to_string())),
        );

        // parse_csi_rxvt_mouse
        assert_eq!(
            parse_event(b"\x1B[32;30;40;M").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 29,
                row: 39,
                modifiers: KeyModifiers::empty(),
            }))
        );

        // parse_csi_normal_mouse
        assert_eq!(
            parse_event(b"\x1B[M0\x60\x70").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 63,
                row: 79,
                modifiers: KeyModifiers::CONTROL,
            }))
        );

        // parse_csi_sgr_mouse
        assert_eq!(
            parse_event(b"\x1B[<0;20;10;M").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 19,
                row: 9,
                modifiers: KeyModifiers::empty(),
            }))
        );

        // parse_utf8_char
        assert_eq!(
            parse_event("Ž".as_bytes()).unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('Ž'),
                KeyModifiers::SHIFT
            ))),
        );
    }

    #[test]
    fn test_parse_tab() {
        assert_eq!(
            parse_event(b"\t").unwrap(),
            Some(Event::Key(KeyCode::Tab.into())),
        );

        assert_eq!(Event::Key(KeyCode::Tab.into()).to_escape_sequence(), b"\t");

        assert_eq!(
            parse_event(b"\x1B[Z").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::BackTab,
                KeyModifiers::SHIFT,
                KeyEventKind::Press,
            ))),
        );

        assert_eq!(
            Event::Key(KeyCode::BackTab.into()).to_escape_sequence(),
            b"\x1B[Z"
        );
    }

    #[test]
    fn test_parse_csi() {
        assert_eq!(
            parse_event(b"\x1B[D").unwrap(),
            Some(Event::Key(KeyCode::Left.into())),
        );
    }

    #[test]
    fn test_parse_csi_modifier_key_code() {
        assert_eq!(
            parse_event(b"\x1B[2D").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::SHIFT
            ))),
        );
    }

    #[test]
    fn test_parse_csi_special_key_code() {
        assert_eq!(
            parse_event(b"\x1B[3~").unwrap(),
            Some(Event::Key(KeyCode::Delete.into())),
        );
    }

    #[test]
    fn test_parse_csi_special_key_code_multiple_values_not_supported() {
        assert_eq!(
            parse_event(b"\x1B[3;2~").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Delete,
                KeyModifiers::SHIFT
            ))),
        );
    }

    #[test]
    fn test_parse_csi_bracketed_paste() {
        //
        assert_eq!(
            parse_event(b"\x1B[200~o").unwrap(),
            None,
            "A partial bracketed paste isn't parsed"
        );
        assert_eq!(
            parse_event(b"\x1B[200~o\x1B[2D").unwrap(),
            None,
            "A partial bracketed paste containing another escape code isn't parsed"
        );
        assert_eq!(
            parse_event(b"\x1B[200~o\x1B[2D\x1B[201~").unwrap(),
            Some(Event::Paste("o\x1B[2D".to_string()))
        );
    }

    #[test]
    fn test_parse_csi_focus() {
        assert_eq!(parse_csi(b"\x1B[O").unwrap(), Some(Event::FocusLost));
    }

    #[test]
    fn test_parse_csi_rxvt_mouse() {
        assert_eq!(
            parse_event(b"\x1B[32;30;40;M").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 29,
                row: 39,
                modifiers: KeyModifiers::empty(),
            }))
        );
    }

    #[test]
    fn test_parse_csi_normal_mouse() {
        assert_eq!(
            parse_event(b"\x1B[M0\x60\x70").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 63,
                row: 79,
                modifiers: KeyModifiers::CONTROL,
            }))
        );
    }

    #[test]
    fn test_parse_csi_sgr_mouse() {
        assert_eq!(
            parse_event(b"\x1B[<0;20;10;M").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 19,
                row: 9,
                modifiers: KeyModifiers::empty(),
            }))
        );
        assert_eq!(
            parse_event(b"\x1B[<0;20;10M").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 19,
                row: 9,
                modifiers: KeyModifiers::empty(),
            }))
        );
        assert_eq!(
            parse_event(b"\x1B[<0;20;10;m").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 19,
                row: 9,
                modifiers: KeyModifiers::empty(),
            }))
        );
        assert_eq!(
            parse_event(b"\x1B[<0;20;10m").unwrap(),
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 19,
                row: 9,
                modifiers: KeyModifiers::empty(),
            }))
        );
    }

    #[test]
    fn test_utf8() {
        // https://www.php.net/manual/en/reference.pcre.pattern.modifiers.php#54805

        // 'Valid ASCII' => "a",
        assert_eq!(
            parse_event(b"a").unwrap(),
            Some(Event::Key(KeyCode::Char('a').into()))
        );

        // 'Valid 2 Octet Sequence' => "\xc3\xb1",
        assert_eq!(
            parse_event(&[0xC3, 0xB1]).unwrap(),
            Some(Event::Key(KeyCode::Char('ñ').into()))
        );

        // 'Invalid 2 Octet Sequence' => "\xc3\x28",
        assert!(parse_event(&[0xC3, 0x28]).is_err());

        // 'Invalid Sequence Identifier' => "\xa0\xa1",
        assert!(parse_event(&[0xA0, 0xA1]).is_err());

        // 'Valid 3 Octet Sequence' => "\xe2\x82\xa1",
        assert_eq!(
            parse_event(&[0xE2, 0x81, 0xA1]).unwrap(),
            Some(Event::Key(KeyCode::Char('\u{2061}').into()))
        );

        // 'Invalid 3 Octet Sequence (in 2nd Octet)' => "\xe2\x28\xa1",
        assert!(parse_event(&[0xE2, 0x28, 0xA1]).is_err());

        // 'Invalid 3 Octet Sequence (in 3rd Octet)' => "\xe2\x82\x28",
        assert!(parse_event(&[0xE2, 0x82, 0x28]).is_err());

        // 'Valid 4 Octet Sequence' => "\xf0\x90\x8c\xbc",
        assert_eq!(
            parse_event(&[0xF0, 0x90, 0x8C, 0xBC]).unwrap(),
            Some(Event::Key(KeyCode::Char('𐌼').into()))
        );

        // 'Invalid 4 Octet Sequence (in 2nd Octet)' => "\xf0\x28\x8c\xbc",
        assert!(parse_event(&[0xF0, 0x28, 0x8C, 0xBC]).is_err());

        // 'Invalid 4 Octet Sequence (in 3rd Octet)' => "\xf0\x90\x28\xbc",
        assert!(parse_event(&[0xF0, 0x90, 0x28, 0xBC]).is_err());

        // 'Invalid 4 Octet Sequence (in 4th Octet)' => "\xf0\x28\x8c\x28",
        assert!(parse_event(&[0xF0, 0x28, 0x8C, 0x28]).is_err());
    }

    #[test]
    fn test_parse_char_event_lowercase() {
        assert_eq!(
            parse_event(b"c").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::empty()
            ))),
        );

        assert_eq!(
            Event::Key(KeyCode::Char('c').into()).to_escape_sequence(),
            b"c"
        );
    }

    #[test]
    fn test_parse_char_event_uppercase() {
        assert_eq!(
            parse_event(b"C").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('C'),
                KeyModifiers::SHIFT
            ))),
        );

        assert_eq!(
            Event::Key(KeyCode::Char('C').into()).to_escape_sequence(),
            b"C"
        );
    }

    #[test]
    fn test_parse_basic_csi_u_encoded_key_code() {
        assert_eq!(
            parse_event(b"\x1B[97u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;2u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('A'),
                KeyModifiers::SHIFT
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;7u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::ALT | KeyModifiers::CONTROL
            ))),
        );
    }

    #[test]
    fn test_parse_fn_keys() {
        assert_eq!(
            parse_event(b"\x1B[11~").unwrap(),
            Some(Event::Key(KeyCode::F(1).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(1).into()).to_escape_sequence(),
            b"\x1B[11~"
        );

        assert_eq!(
            parse_event(b"\x1B[12~").unwrap(),
            Some(Event::Key(KeyCode::F(2).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(2).into()).to_escape_sequence(),
            b"\x1B[12~"
        );

        assert_eq!(
            parse_event(b"\x1B[13~").unwrap(),
            Some(Event::Key(KeyCode::F(3).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(3).into()).to_escape_sequence(),
            b"\x1B[13~"
        );

        assert_eq!(
            parse_event(b"\x1B[14~").unwrap(),
            Some(Event::Key(KeyCode::F(4).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(4).into()).to_escape_sequence(),
            b"\x1B[14~"
        );

        assert_eq!(
            parse_event(b"\x1B[15~").unwrap(),
            Some(Event::Key(KeyCode::F(5).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(5).into()).to_escape_sequence(),
            b"\x1B[15~"
        );

        assert_eq!(
            parse_event(b"\x1B[17~").unwrap(),
            Some(Event::Key(KeyCode::F(6).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(6).into()).to_escape_sequence(),
            b"\x1B[17~"
        );

        assert_eq!(
            parse_event(b"\x1B[18~").unwrap(),
            Some(Event::Key(KeyCode::F(7).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(7).into()).to_escape_sequence(),
            b"\x1B[18~"
        );

        assert_eq!(
            parse_event(b"\x1B[19~").unwrap(),
            Some(Event::Key(KeyCode::F(8).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(8).into()).to_escape_sequence(),
            b"\x1B[19~"
        );

        assert_eq!(
            parse_event(b"\x1B[20~").unwrap(),
            Some(Event::Key(KeyCode::F(9).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(9).into()).to_escape_sequence(),
            b"\x1B[20~"
        );

        assert_eq!(
            parse_event(b"\x1B[21~").unwrap(),
            Some(Event::Key(KeyCode::F(10).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(10).into()).to_escape_sequence(),
            b"\x1B[21~"
        );

        assert_eq!(
            parse_event(b"\x1B[23~").unwrap(),
            Some(Event::Key(KeyCode::F(11).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(11).into()).to_escape_sequence(),
            b"\x1B[23~"
        );

        assert_eq!(
            parse_event(b"\x1B[24~").unwrap(),
            Some(Event::Key(KeyCode::F(12).into())),
        );
        assert_eq!(
            Event::Key(KeyCode::F(12).into()).to_escape_sequence(),
            b"\x1B[24~"
        );
    }

    #[test]
    fn test_parse_basic_csi_u_encoded_key_code_special_keys() {
        assert_eq!(
            parse_event(b"\x1B[13u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[27u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::empty()
            ))),
        );

        assert_eq!(
            parse_event(b"\x1B[57358u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::CapsLock,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::CapsLock, KeyModifiers::empty()))
                .to_escape_sequence(),
            b"\x1B[57358u"
        );

        assert_eq!(
            parse_event(b"\x1B[57359u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::ScrollLock,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::ScrollLock, KeyModifiers::empty()))
                .to_escape_sequence(),
            b"\x1B[57359u"
        );

        assert_eq!(
            parse_event(b"\x1B[57360u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::NumLock,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::NumLock, KeyModifiers::empty())).to_escape_sequence(),
            b"\x1B[57360u"
        );

        assert_eq!(
            parse_event(b"\x1B[57361u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::PrintScreen,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::PrintScreen, KeyModifiers::empty()))
                .to_escape_sequence(),
            b"\x1B[57361u"
        );

        assert_eq!(
            parse_event(b"\x1B[57362u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Pause,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Pause, KeyModifiers::empty())).to_escape_sequence(),
            b"\x1B[57362u"
        );

        assert_eq!(
            parse_csi_u_encoded_key_code(b"\x1B[57363u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Menu,
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            Event::Key(KeyEvent::new(KeyCode::Menu, KeyModifiers::empty())).to_escape_sequence(),
            b"\x1B[57363u"
        );

        assert_eq!(
            parse_event(b"\x1B[57376u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::F(13),
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57428u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Media(MediaKeyCode::Play),
                KeyModifiers::empty()
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57441u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Modifier(ModifierKeyCode::Shift),
                KeyModifiers::SHIFT,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_encoded_keypad_code() {
        assert_eq!(
            parse_event(b"\x1B[57399u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind_and_state(
                KeyCode::Char('0'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
                KeyEventState::KEYPAD,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57419u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind_and_state(
                KeyCode::Up,
                KeyModifiers::empty(),
                KeyEventKind::Press,
                KeyEventState::KEYPAD,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_encoded_key_code_with_types() {
        assert_eq!(
            parse_event(b"\x1B[97;1u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;1:1u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;5:1u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL,
                KeyEventKind::Press,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;1:2u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::empty(),
                KeyEventKind::Repeat,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;1:3u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('a'),
                KeyModifiers::empty(),
                KeyEventKind::Release,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_encoded_key_code_has_modifier_on_modifier_press() {
        assert_eq!(
            parse_event(b"\x1B[57449u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Modifier(ModifierKeyCode::Alt),
                KeyModifiers::ALT,
                KeyEventKind::Press,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57449;3:3u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Modifier(ModifierKeyCode::Alt),
                KeyModifiers::ALT,
                KeyEventKind::Release,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57450u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Modifier(ModifierKeyCode::Super),
                KeyModifiers::SUPER,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57451u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Modifier(ModifierKeyCode::Hyper),
                KeyModifiers::HYPER,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[57452u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Modifier(ModifierKeyCode::Meta),
                KeyModifiers::META,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_encoded_key_code_with_extra_modifiers() {
        assert_eq!(
            parse_event(b"\x1B[97;9u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::SUPER
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;17u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::HYPER,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[97;33u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::META,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_encoded_key_code_with_extra_state() {
        assert_eq!(
            parse_event(b"\x1B[97;65u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind_and_state(
                KeyCode::Char('a'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
                KeyEventState::CAPS_LOCK,
            ))),
        );
        assert_eq!(
            parse_csi_u_encoded_key_code(b"\x1B[49;129u").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind_and_state(
                KeyCode::Char('1'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
                KeyEventState::NUM_LOCK,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_u_with_shifted_keycode() {
        assert_eq!(
            // A-S-9 is equivalent to A-(
            parse_event(b"\x1B[57:40;4u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('('),
                KeyModifiers::ALT,
            ))),
        );
        assert_eq!(
            // A-S-minus is equivalent to A-_
            parse_event(b"\x1B[45:95;4u").unwrap(),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('_'),
                KeyModifiers::ALT,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_special_key_code_with_types() {
        assert_eq!(
            parse_event(b"\x1B[;1:3B").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Down,
                KeyModifiers::empty(),
                KeyEventKind::Release,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[1;1:3B").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::Down,
                KeyModifiers::empty(),
                KeyEventKind::Release,
            ))),
        );
    }

    #[test]
    fn test_parse_csi_numbered_escape_code_with_types() {
        assert_eq!(
            parse_event(b"\x1B[5;1:1~").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::PageUp,
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ))),
        );

        assert_eq!(
            Event::Key(KeyEvent::new_with_kind(
                KeyCode::PageUp,
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ))
            .to_escape_sequence(),
            b"\x1B[5;1:1~"
        );

        assert_eq!(
            parse_event(b"\x1B[5;1:3~").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::PageUp,
                KeyModifiers::empty(),
                KeyEventKind::Release,
            ))),
        );
        assert_eq!(
            parse_event(b"\x1B[6;5:3~").unwrap(),
            Some(Event::Key(KeyEvent::new_with_kind(
                KeyCode::PageDown,
                KeyModifiers::CONTROL,
                KeyEventKind::Release,
            ))),
        );

        assert_eq!(
            Event::Key(KeyEvent::new_with_kind(
                KeyCode::PageDown,
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ))
            .to_escape_sequence(),
            b"\x1B[6;1:1~"
        );
    }
}
