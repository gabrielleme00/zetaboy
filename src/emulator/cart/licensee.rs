pub fn old_name(code: u8) -> &'static str {
    match code {
        0x00 => "None",
        0x01 => "Nintendo",
        0x08 => "Capcom",
        0x09 => "HOT-B",
        0x0A => "Jaleco",
        0x0B => "Coconuts",
        0x0C => "Elite Systems",
        0x13 => "Electronic Arts",
        0x18 => "Hudson Soft",
        0x19 => "ITC Entertainment",
        0x1A => "Yanoman",
        0x1D => "Clary",
        0x1F => "Virgin",
        0x24 => "PCM Complete",
        0x25 => "San-X",
        0x28 => "Kotobuki Systems",
        0x29 => "Seta",
        0x30 => "Infogrames",
        0x31 => "Nintendo",
        0x32 => "Bandai",
        0x34 => "Konami",
        0x35 => "Hector",
        0x38 => "Capcom",
        0x39 => "Banpresto",
        0x41 => "Ubi Soft",
        0x42 => "Atlus",
        0x44 => "Malibu",
        0x46 => "Angel",
        0x47 => "Spectrum Holoby",
        0x49 => "Irem",
        0x4A => "Virgin",
        0x4D => "Malibu",
        0x4F => "U.S. Gold",
        0x50 => "Absolute",
        0x51 => "Acclaim",
        0x52 => "Activision",
        0x53 => "American sammy",
        0x54 => "GameTek",
        0x55 => "Park Place",
        0x56 => "LJN",
        0x57 => "Matchbox",
        0x58 => "Mattel",
        0x59 => "Milton Bradley",
        0x5A => "Mindscape",
        0x5B => "Romstar",
        0x5C => "Naxat Soft",
        0x5D => "Tradewest",
        0x60 => "Titus",
        0x61 => "Virgin",
        0x67 => "Ocean",
        0x69 => "Electronic Arts",
        0x6E => "Elite Systems",
        0x6F => "Electro Brain",
        0x70 => "Infogrames",
        0x71 => "Interplay",
        0x72 => "Broderbund",
        0x73 => "Sculptured Soft",
        0x75 => "The Sales Curve",
        0x78 => "THQ",
        0x79 => "Accolade",
        0x7A => "Triffix Entertainment",
        0x7C => "Micropose",
        0x7F => "Kemco",
        0x80 => "Misawa Entertainment",
        0x83 => "lozc",
        0x86 => "Tokuma Shoten Intermedia",
        0x8B => "Bullet-Proof Software",
        0x8C => "Vic Tokai",
        0x8E => "Ape",
        0x8F => "iMAX",
        0x91 => "Chunsoft",
        0x92 => "Video system",
        0x93 => "Tsuburava",
        0x95 => "Varie",
        0x96 => "Yonezawa/s’pal",
        0x97 => "Kaneko",
        0x99 => "Arc",
        0x9A => "Nihon Bussan",
        0x9B => "Tecmo",
        0x9C => "imagineer",
        0x9D => "banpresto",
        0x9F => "nova",
        0xA1 => "hori electric",
        0xA2 => "bandai" ,
        0xA4 => "konami",
        0xA6 => "kawada" ,
        0xA7 => "takara" ,
        0xA9 => "technos japan",
        0xAA => "broderbund" ,
        0xAC => "toei animation",
        0xAD => "toho",
        0xAF => "namco",
        0xB0 => "acclaim",
        0xB1 => "ascii or nexoft",
        0xB2 => "bandai" ,
        0xB4 => "enix" ,
        0xB6 => "hal",
        0xB7 => "snk",
        0xB9 => "pony canyon",
        0xBA => "*culture brain o",
        0xBB => "sunsoft",
        0xBD => "sony imagesoft",
        0xBF => "sammy",
        0xC0 => "taito",
        0xC2 => "kemco",
        0xC3 => "squaresoft",
        0xC4 => "*tokuma shoten i" ,
        0xC5 => "data east",
        0xC6 => "tonkin house",
        0xC8 => "koei" ,
        0xC9 => "ufl",
        0xCA => "ultra",
        0xCB => "vap",
        0xCC => "use",
        0xCD => "meldac",
        0xCE => "*pony canyon or",
        0xCF => "angel",
        0xD0 => "taito",
        0xD1 => "sofel",
        0xD2 => "quest",
        0xD3 => "sigma enterprises",
        0xD4 => "ask kodansha" ,
        0xD6 => "naxat soft",
        0xD7 => "copya systems",
        0xD9 => "banpresto",
        0xDA => "tomy",
        0xDB => "ljn",
        0xDD => "ncs",
        0xDE => "human",
        0xDF => "altron",
        0xE0 => "jaleco" ,
        0xE1 => "towachiki",
        0xE2 => "uutaka",
        0xE3 => "varie",
        0xE5 => "epoch",
        0xE7 => "athena",
        0xE8 => "asmik",
        0xE9 => "natsume",
        0xEA => "king records",
        0xEB => "atlus",
        0xEC => "epic/sony records",
        0xEE => "igs",
        0xF0 => "a wave" ,
        0xF3 => "extreme entertainment",
        0xFF => "ljn",
        _ => "UNKNOWN",
    }
}

pub fn new_name(code: u16) -> &'static str {
    match code {
        0x00 => "None",
        0x01 => "Nintendo R&D1",
        0x08 => "Capcom",
        0x13 => "Electronic Arts",
        0x18 => "Hudson Soft",
        0x19 => "b-ai",
        0x20 => "kss",
        0x22 => "pow",
        0x24 => "PCM Complete",
        0x25 => "San-X",
        0x28 => "Kemco Japan",
        0x29 => "seta",
        0x30 => "Viacom",
        0x31 => "Nintendo",
        0x32 => "Bandai",
        0x33 => "Ocean/Acclaim",
        0x34 => "Konami",
        0x35 => "Hector",
        0x37 => "Taito",
        0x38 => "Hudson",
        0x39 => "Banpresto",
        0x41 => "Ubi Soft",
        0x42 => "Atlus",
        0x44 => "Malibu",
        0x46 => "angel",
        0x47 => "Bullet-Proof",
        0x49 => "irem",
        0x50 => "Absolute",
        0x51 => "Acclaim",
        0x52 => "Activision",
        0x53 => "American sammy",
        0x54 => "Konami",
        0x55 => "Hi tech entertainment",
        0x56 => "LJN",
        0x57 => "Matchbox",
        0x58 => "Mattel",
        0x59 => "Milton Bradley",
        0x60 => "Titus",
        0x61 => "Virgin",
        0x64 => "LucasArts",
        0x67 => "Ocean",
        0x69 => "Electronic Arts",
        0x70 => "Infogrames",
        0x71 => "Interplay",
        0x72 => "Broderbund",
        0x73 => "sculptured",
        0x75 => "sci",
        0x78 => "THQ",
        0x79 => "Accolade",
        0x80 => "misawa",
        0x83 => "lozc",
        0x86 => "Tokuma Shoten Intermedia",
        0x87 => "Tsukuda Original",
        0x91 => "Chunsoft",
        0x92 => "Video system",
        0x93 => "Ocean/Acclaim",
        0x95 => "Varie",
        0x96 => "Yonezawa/s’pal",
        0x97 => "Kaneko",
        0x99 => "Pack in soft",
        0xA4 => "Konami (Yu-Gi-Oh!)",
        _ => "UNKNOWN",
    }
}
