#![allow(dead_code)]
// Set the tempo in beats per minute (BPM)
const TEMPO: f32 = 114.0;

// Define a time basis of a 1/16 note in ms
pub const BASIS: f32 = (60.0 / TEMPO) * 1000.0 / 4.0;

#[derive(PartialEq, Clone, Copy)]
pub enum Pitch {
    REST = 0,
    C4 = 262,
    CS4 = 277,
    D4 = 294,
    DS4 = 311,
    E4 = 330,
    F4 = 349,
    FS4 = 370,
    G4 = 392,
    GS4 = 415,
    A4 = 440,
    AS4 = 466,
    B4 = 494,
    C5 = 523,
    CS5 = 554,
    D5 = 587,
}

pub struct Note {
    pub pitch: Pitch,
    pub duration: u32, // Duration in terms of 1/16 notes
}

#[rustfmt::skip]
pub const RICKROLL: [Note; 12] = [
    Note { pitch: Pitch::G4, duration: 6 },
    Note { pitch: Pitch::A4, duration: 6 },
    Note { pitch: Pitch::D4, duration: 4 },

    Note { pitch: Pitch::A4, duration: 6 },
    Note { pitch: Pitch::B4, duration: 6 },
    Note { pitch: Pitch::D5, duration: 1 },
    Note { pitch: Pitch::C5, duration: 1 },
    Note { pitch: Pitch::B4, duration: 2 },

    Note { pitch: Pitch::G4, duration: 6 },
    Note { pitch: Pitch::A4, duration: 6 },
    Note { pitch: Pitch::D4, duration: 10 },

    Note { pitch: Pitch::REST, duration: 10 },    
];
