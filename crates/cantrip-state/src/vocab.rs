//! The game's vocabulary as enums and bitsets: nothing here is a string at runtime.

use std::fmt;

/// A player, by seat: 0 is the first seat (Bot1 in a scenario).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// An object (CR 109). An object that changes zones becomes a new object with a new id (CR 400.7); the old row stays
/// behind, frozen, as its last-known information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ObjectId(pub u32);

impl ObjectId {
    /// No object: the source of mana a player's own mana ability adds (CR 605.1a: such an ability has no source object).
    pub const NONE: ObjectId = ObjectId(u32::MAX);

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// A card in the immutable card table, built once per process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CardId(pub u16);

/// How a spell cast as its Adventure is cast (CR 715.3), as a way number: after every alternative cost.
pub const WAY_ADVENTURE: u8 = 254;

/// How a spell cast as its Omen is cast (CR 720.3), as a way number: beside the Adventure, after every alternative cost.
pub const WAY_OMEN: u8 = 249;

/// How a modal double-faced card is cast as its back face (CR 712.2, 712.8a), as a way number: beside the front face's
/// casts, after every alternative cost.
pub const WAY_BACK_FACE: u8 = 253;

/// How a card with bestow is cast bestowed (CR 702.103a), as a way number: beside the other casts, after every alternative
/// cost.
pub const WAY_BESTOW: u8 = 251;

/// How a card in a graveyard is cast by a way a permanent on the battlefield gives it (CR 113.6, 118.9), as a way number:
/// `WAY_GRANTED + k * WAY_GRANTED_STRIDE + face` for the kth way the battlefield grants, in the order the permanents and
/// their rules are listed, and the face of the card it is cast as (`GRANTED_FACE_*`): a way is judged on the face it
/// casts, never on the card as a whole (CR 709.3, 715.3, 712.8a). The band lies below the special ways and above any a card
/// prints; a cast decodes it and goes on as that face's way (0, a split half, the Adventure, the back face, the Omen).
pub const WAY_GRANTED: u8 = 128;
/// How many ways a battlefield may grant at once, and how many numbers each takes.
pub const WAY_GRANTED_SLOTS: u8 = 8;
pub const WAY_GRANTED_STRIDE: u8 = 6;
/// The faces a granted cast names: the card itself, a split card's left half and right half, the Adventure, the back face,
/// the Omen.
pub const GRANTED_FACE_WHOLE: u8 = 0;
pub const GRANTED_FACE_LEFT: u8 = 1;
pub const GRANTED_FACE_RIGHT: u8 = 2;
pub const GRANTED_FACE_ADVENTURE: u8 = 3;
pub const GRANTED_FACE_BACK: u8 = 4;
pub const GRANTED_FACE_OMEN: u8 = 5;

/// The way number of the kth granted way cast as `face`.
pub fn granted_way(k: usize, face: u8) -> u8 {
    WAY_GRANTED + k as u8 * WAY_GRANTED_STRIDE + face
}

/// The granted way (its index among them) and the face a way number names, or `None` for a way that is not granted.
pub fn granted_slot(way: u8) -> Option<(usize, u8)> {
    let n = way.checked_sub(WAY_GRANTED)?;
    (n < WAY_GRANTED_SLOTS * WAY_GRANTED_STRIDE).then(|| ((n / WAY_GRANTED_STRIDE) as usize, n % WAY_GRANTED_STRIDE))
}

/// The way number a cast of a face goes on as once it has left its granted way behind.
pub fn granted_face_way(face: u8) -> u8 {
    match face {
        GRANTED_FACE_LEFT => WAY_SPLIT_LEFT,
        GRANTED_FACE_RIGHT => WAY_SPLIT_RIGHT,
        GRANTED_FACE_ADVENTURE => WAY_ADVENTURE,
        GRANTED_FACE_BACK => WAY_BACK_FACE,
        GRANTED_FACE_OMEN => WAY_OMEN,
        _ => 0,
    }
}

/// How a card with overload is cast overloaded (CR 702.96a), as a way number: for its overload cost, with "each" where it
/// says "target", so with no targets at all.
pub const WAY_OVERLOAD: u8 = 250;

/// How the halves of a split card are cast (CR 709.3), as way numbers: the left half and the right.
pub const WAY_SPLIT_LEFT: u8 = 239;
pub const WAY_SPLIT_RIGHT: u8 = 238;

/// The doors of a Room (CR 709.5, 709.5j), as the bits of `Marks::doors`: the left half and the right.
pub const DOOR_LEFT: u8 = 1;
pub const DOOR_RIGHT: u8 = 2;

/// How a card is cast transformed (CR 712.11a): a Siege's "cast it transformed", as a way number.
pub const WAY_TRANSFORMED: u8 = 252;

/// One ability of one card: the card's definition and the ability's index in it. A copy of a card has the copied
/// card's abilities, so a computed ability list is a list of these, never of the object's own card alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AbilityRef {
    pub card: CardId,
    pub index: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Blue = 1,
    Black = 2,
    Red = 3,
    Green = 4,
}

impl Color {
    pub const ALL: [Color; 5] = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];

    pub fn name(self) -> &'static str {
        match self {
            Color::White => "White",
            Color::Blue => "Blue",
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Black => 'B',
            Color::Red => 'R',
            Color::Green => 'G',
        }
    }

    pub fn mana(self) -> Mana {
        match self {
            Color::White => Mana::W,
            Color::Blue => Mana::U,
            Color::Black => Mana::B,
            Color::Red => Mana::R,
            Color::Green => Mana::G,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ColorSet(pub u8);

impl ColorSet {
    pub const NONE: ColorSet = ColorSet(0);

    pub const fn of(color: Color) -> ColorSet {
        ColorSet(1 << color as u8)
    }

    pub fn has(self, color: Color) -> bool {
        self.0 & (1 << color as u8) != 0
    }

    pub fn with(self, color: Color) -> ColorSet {
        ColorSet(self.0 | (1 << color as u8))
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn intersects(self, other: ColorSet) -> bool {
        self.0 & other.0 != 0
    }

    pub fn iter(self) -> impl Iterator<Item = Color> {
        Color::ALL.into_iter().filter(move |c| self.has(*c))
    }
}

impl fmt::Debug for ColorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// A kind of mana (CR 106): the five colors and colorless.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Mana {
    W = 0,
    U = 1,
    B = 2,
    R = 3,
    G = 4,
    C = 5,
}

impl Mana {
    pub const ALL: [Mana; 6] = [Mana::W, Mana::U, Mana::B, Mana::R, Mana::G, Mana::C];

    pub fn color(self) -> Option<Color> {
        match self {
            Mana::W => Some(Color::White),
            Mana::U => Some(Color::Blue),
            Mana::B => Some(Color::Black),
            Mana::R => Some(Color::Red),
            Mana::G => Some(Color::Green),
            Mana::C => None,
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Mana::W => 'W',
            Mana::U => 'U',
            Mana::B => 'B',
            Mana::R => 'R',
            Mana::G => 'G',
            Mana::C => 'C',
        }
    }
}

/// Mana in a pool, by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ManaPool(pub [u8; 6]);

impl ManaPool {
    pub fn get(&self, mana: Mana) -> u8 {
        self.0[mana as usize]
    }

    pub fn total(&self) -> u32 {
        self.0.iter().map(|&n| n as u32).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

/// A mana cost (CR 202): generic, and a count of each colored symbol. Enough for the cards so far; hybrid,
/// Phyrexian and X arrive with the first cards that print them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ManaCost {
    pub generic: u8,
    /// W U B R G C, as `Mana` orders them.
    pub symbols: [u8; 6],
    /// Hybrid symbols (CR 107.4e): each paid with either of two colors.
    pub hybrid: [Option<(Mana, Mana)>; 6],
    /// Phyrexian symbols (CR 107.4f) of each color, W U B R G: each paid with that color or 2 life.
    pub phyrexian: [u8; 5],
    /// How many {X} symbols it has (CR 107.3): {X}{X} is 2.
    pub x: u8,
}

impl ManaCost {
    pub const ZERO: ManaCost = ManaCost { generic: 0, symbols: [0; 6], hybrid: [None; 6], phyrexian: [0; 5], x: 0 };

    pub fn mana_value(&self) -> u32 {
        self.generic as u32
            + self.symbols.iter().map(|&n| n as u32).sum::<u32>()
            + self.hybrid.iter().flatten().count() as u32
            + self.phyrexian.iter().map(|&n| n as u32).sum::<u32>()
    }

    pub fn is_zero(&self) -> bool {
        self.mana_value() == 0
    }

    /// The cost as a player who "may spend mana as though it were mana of any color" pays it (CR 609.4b): each colored
    /// symbol and each hybrid symbol of two colors may be paid with any mana, colorless included (Mycosynth Lattice's
    /// ruling), so it is paid as generic; a {C} symbol still takes colorless mana, which no color stands for, and a
    /// Phyrexian symbol keeps its 2-life choice (announced before the mana is paid, which turns it into its color's
    /// symbol, and that then any mana pays). The cost itself, and its mana value, are not changed (CR 609.4b): this is the
    /// payment's view of it only.
    pub fn paid_with_any_color(&self) -> ManaCost {
        let mut paid = *self;
        let mut generic = self.generic as u32;
        for i in 0..5 {
            generic += self.symbols[i] as u32;
            paid.symbols[i] = 0;
        }
        // A hybrid symbol has a colored half, which any mana may now pay.
        generic += paid.hybrid.iter().flatten().count() as u32;
        paid.hybrid = [None; 6];
        paid.generic = generic.min(255) as u8;
        paid
    }

    /// The two costs added symbol by symbol (CR 709.4b: a split card's mana cost is its halves' combined; a Room's
    /// is that of its unlocked doors, CR 709.5).
    pub fn plus(&self, other: &ManaCost) -> ManaCost {
        let mut sum = *self;
        sum.generic = sum.generic.saturating_add(other.generic);
        sum.x = sum.x.saturating_add(other.x);
        for i in 0..6 {
            sum.symbols[i] = sum.symbols[i].saturating_add(other.symbols[i]);
        }
        for i in 0..5 {
            sum.phyrexian[i] = sum.phyrexian[i].saturating_add(other.phyrexian[i]);
        }
        for pair in other.hybrid.iter().flatten() {
            if let Some(slot) = sum.hybrid.iter().position(|h| h.is_none()) {
                sum.hybrid[slot] = Some(*pair);
            }
        }
        sum
    }

    pub fn colors(&self) -> ColorSet {
        let mut set = ColorSet::NONE;
        for color in Color::ALL {
            if self.symbols[color.mana() as usize] > 0 {
                set = set.with(color);
            }
        }
        for (a, b) in self.hybrid.iter().flatten() {
            for m in [a, b] {
                if let Some(c) = m.color() {
                    set = set.with(c);
                }
            }
        }
        for (i, &n) in self.phyrexian.iter().enumerate() {
            if n > 0 {
                set = set.with(Color::ALL[i]);
            }
        }
        set
    }
}

impl fmt::Display for ManaCost {
    /// As the C# engine writes a cost: `{2}{R}`, and `{0}` for nothing.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.x {
            f.write_str("{X}")?;
        }
        if self.generic > 0 || self.mana_value() == 0 && self.x == 0 {
            write!(f, "{{{}}}", self.generic)?;
        }
        for (a, b) in self.hybrid.iter().flatten() {
            write!(f, "{{{}/{}}}", a.symbol(), b.symbol())?;
        }
        for (i, &n) in self.phyrexian.iter().enumerate() {
            for _ in 0..n {
                write!(f, "{{{}/P}}", Color::ALL[i].symbol())?;
            }
        }
        for mana in Mana::ALL {
            for _ in 0..self.symbols[mana as usize] {
                write!(f, "{{{}}}", mana.symbol())?;
            }
        }
        Ok(())
    }
}

/// Card types (CR 300), as a bitset.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TypeSet(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CardType {
    Artifact = 0,
    Creature = 1,
    Enchantment = 2,
    Instant = 3,
    Land = 4,
    Planeswalker = 5,
    Sorcery = 6,
    Battle = 7,
    Kindred = 8,
}

impl CardType {
    pub const ALL: [CardType; 9] = [
        CardType::Kindred,
        CardType::Artifact,
        CardType::Battle,
        CardType::Creature,
        CardType::Enchantment,
        CardType::Instant,
        CardType::Land,
        CardType::Planeswalker,
        CardType::Sorcery,
    ];

    pub fn name(self) -> &'static str {
        match self {
            CardType::Artifact => "Artifact",
            CardType::Creature => "Creature",
            CardType::Enchantment => "Enchantment",
            CardType::Instant => "Instant",
            CardType::Land => "Land",
            CardType::Planeswalker => "Planeswalker",
            CardType::Sorcery => "Sorcery",
            CardType::Battle => "Battle",
            CardType::Kindred => "Kindred",
        }
    }
}

impl TypeSet {
    pub const NONE: TypeSet = TypeSet(0);

    pub const fn of(types: &[CardType]) -> TypeSet {
        let mut bits = 0u16;
        let mut i = 0;
        while i < types.len() {
            bits |= 1 << types[i] as u8;
            i += 1;
        }
        TypeSet(bits)
    }

    pub fn has(self, t: CardType) -> bool {
        self.0 & (1 << t as u8) != 0
    }

    pub fn with(self, t: CardType) -> TypeSet {
        TypeSet(self.0 | (1 << t as u8))
    }

    pub fn iter(self) -> impl Iterator<Item = CardType> {
        CardType::ALL.into_iter().filter(move |t| self.has(*t))
    }
}

impl fmt::Debug for TypeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// The qualities of "hexproof from [quality]" abilities (CR 702.11d): a permanent with one can't be the target of
/// [quality] spells its controller's opponents control, nor of abilities they control from [quality] sources. A bit
/// for each quality: the five colors (`ColorSet`'s bits), monocolored and multicolored (CR 105.2a-b), and card types
/// (`TypeSet`'s bits, from bit 8: "hexproof from artifacts", "from instants"). "Hexproof from [A] and from [B]" is two
/// of them (CR 702.11f), "hexproof from each color" one for each color (CR 702.11g), and the same one twice is
/// redundant (CR 702.11h).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct HexproofFrom(pub u32);

impl HexproofFrom {
    pub const NONE: HexproofFrom = HexproofFrom(0);
    /// "Hexproof from monocolored": sources that are exactly one color (CR 105.2a).
    pub const MONOCOLORED: HexproofFrom = HexproofFrom(1 << 5);
    /// "Hexproof from multicolored": sources that are two or more colors (CR 105.2b).
    pub const MULTICOLORED: HexproofFrom = HexproofFrom(1 << 6);
    /// "Hexproof from each color" (CR 702.11g).
    pub const EACH_COLOR: HexproofFrom = HexproofFrom(0x1f);
    const TYPES_SHIFT: u32 = 8;

    /// "Hexproof from [color]".
    pub const fn color(color: Color) -> HexproofFrom {
        HexproofFrom(1 << color as u8)
    }

    /// "Hexproof from [card type]s": sources of that card type.
    pub const fn card_type(t: CardType) -> HexproofFrom {
        HexproofFrom(1 << (t as u32 + Self::TYPES_SHIFT))
    }

    /// Both sets of qualities (CR 702.11f).
    pub const fn and(self, other: HexproofFrom) -> HexproofFrom {
        HexproofFrom(self.0 | other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The colors it names.
    pub fn colors(self) -> ColorSet {
        ColorSet((self.0 & 0x1f) as u8)
    }

    /// These qualities with the colors it names replaced by `colors`: a text-changing effect's new color word (CR 612.2).
    pub fn with_colors(self, colors: ColorSet) -> HexproofFrom {
        HexproofFrom((self.0 & !0x1f) | colors.0 as u32)
    }

    /// The card types it names.
    pub fn types(self) -> TypeSet {
        TypeSet((self.0 >> Self::TYPES_SHIFT) as u16)
    }

    pub fn has(self, quality: HexproofFrom) -> bool {
        self.0 & quality.0 == quality.0 && quality.0 != 0
    }

    /// Whether a source with these colors and card types has one of the qualities (CR 702.11d): one of the colors, one
    /// color only (monocolored) or two or more (multicolored), one of the card types. A colorless source is none of them.
    pub fn covers(self, colors: ColorSet, types: TypeSet) -> bool {
        let n = colors.0.count_ones();
        self.colors().intersects(colors)
            || (self.has(HexproofFrom::MONOCOLORED) && n == 1)
            || (self.has(HexproofFrom::MULTICOLORED) && n >= 2)
            || self.types().0 & types.0 != 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Supertypes {
    pub basic: bool,
    pub legendary: bool,
    pub snow: bool,
}

pub use crate::subtypes::{SUBTYPE_WORDS, Subtype};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SubtypeSet(pub [u64; SUBTYPE_WORDS]);

impl SubtypeSet {
    pub const NONE: SubtypeSet = SubtypeSet([0; SUBTYPE_WORDS]);
    /// Every creature type (CR 205.3m): what Changeling makes an object (CR 702.73a), and what "all creature types"
    /// gives one (Mutavault).
    pub const CREATURE_TYPES: SubtypeSet = SubtypeSet::of(&Subtype::CREATURE_TYPES);
    /// Every land type (CR 205.3i): what an effect that sets a land's subtype replaces (CR 305.7).
    pub const LAND_TYPES: SubtypeSet = SubtypeSet::of(&[
        Subtype::Cave,
        Subtype::Desert,
        Subtype::Forest,
        Subtype::Gate,
        Subtype::Island,
        Subtype::Lair,
        Subtype::Locus,
        Subtype::Mine,
        Subtype::Mountain,
        Subtype::Plains,
        Subtype::Planet,
        Subtype::PowerPlant,
        Subtype::Sphere,
        Subtype::Swamp,
        Subtype::Tower,
        Subtype::Town,
        Subtype::Urzas,
    ]);

    pub const fn of(subtypes: &[Subtype]) -> SubtypeSet {
        let mut bits = [0u64; SUBTYPE_WORDS];
        let mut i = 0;
        while i < subtypes.len() {
            let n = subtypes[i] as usize;
            bits[n / 64] |= 1 << (n % 64);
            i += 1;
        }
        SubtypeSet(bits)
    }

    pub fn has(self, s: Subtype) -> bool {
        let n = s as usize;
        self.0[n / 64] & (1 << (n % 64)) != 0
    }

    pub fn with(self, s: Subtype) -> SubtypeSet {
        let mut bits = self.0;
        let n = s as usize;
        bits[n / 64] |= 1 << (n % 64);
        SubtypeSet(bits)
    }

    pub fn union(self, other: SubtypeSet) -> SubtypeSet {
        let mut bits = self.0;
        for (a, b) in bits.iter_mut().zip(other.0) {
            *a |= b;
        }
        SubtypeSet(bits)
    }

    pub fn is_empty(self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    /// These subtypes but none of `other`'s.
    pub fn minus(self, other: SubtypeSet) -> SubtypeSet {
        let mut bits = self.0;
        for (a, b) in bits.iter_mut().zip(other.0) {
            *a &= !b;
        }
        SubtypeSet(bits)
    }

    pub fn without(self, s: Subtype) -> SubtypeSet {
        let mut bits = self.0;
        let n = s as usize;
        bits[n / 64] &= !(1 << (n % 64));
        SubtypeSet(bits)
    }

    pub fn iter(self) -> impl Iterator<Item = Subtype> {
        Subtype::ALL.into_iter().filter(move |s| self.has(*s))
    }
}

/// A word a text-changing effect can replace (CR 612.2): a color word, or a subtype used as a subtype - a basic land
/// type or a creature type. Mana symbols are not words, and a name is never changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Word {
    Color(Color),
    Subtype(Subtype),
}

/// The words an object's text has now, against the words printed on it (CR 612): the text-changing effects on it in
/// timestamp order, each replacing every instance of one word with another. Card code names a word in its rules text
/// through these (`Query::words`), so "a blue spell" asks `words.color(Color::Blue)` and reads red once that word was
/// changed to red.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Words {
    changes: [Option<(Word, Word)>; Words::MAX],
}

impl Words {
    /// More text changes than any object gets; past it, the rest are not kept.
    pub const MAX: usize = 4;
    pub const NONE: Words = Words { changes: [None; Words::MAX] };

    /// Whether the text is as printed.
    pub fn is_printed(&self) -> bool {
        self.changes[0].is_none()
    }

    /// The text with every instance of `from` replaced by `to` (CR 612.1), after the changes made so far.
    pub fn changed(mut self, from: Word, to: Word) -> Words {
        if let Some(slot) = self.changes.iter().position(|c| c.is_none()) {
            self.changes[slot] = Some((from, to));
        }
        self
    }

    /// Every change of `other` after these.
    pub fn then(self, other: Words) -> Words {
        other.changes.into_iter().flatten().fold(self, |w, (from, to)| w.changed(from, to))
    }

    /// The word printed as `printed`, as the text has it now. The changes apply in timestamp order (CR 613.7) unless one
    /// depends on another (CR 613.8a): a change of "Goblin" depends on a later change that makes a word "Goblin", as
    /// applying that one changes what the first does, so it waits for it (613.8b). For one word that is: the earliest change
    /// not yet applied whose word it is now, again and again, each change once. Two changes of the same word each take the
    /// other's word away, a loop applied in timestamp order: the earlier.
    pub fn word(&self, printed: Word) -> Word {
        let mut word = printed;
        let mut applied = [false; Words::MAX];
        while let Some(i) = (0..Words::MAX).find(|&i| !applied[i] && self.changes[i].is_some_and(|(from, _)| from == word)) {
            applied[i] = true;
            if let Some((_, to)) = self.changes[i] {
                word = to;
            }
        }
        word
    }

    /// The color word printed as `printed`, as the text has it now: "destroy target blue permanent" says `color(Blue)`.
    pub fn color(&self, printed: Color) -> Color {
        match self.word(Word::Color(printed)) {
            Word::Color(c) => c,
            Word::Subtype(_) => printed,
        }
    }

    /// Each color word of `printed`, as the text has it now: "a 1/1 white and black Inkling" made under changed text.
    pub fn colors(&self, printed: ColorSet) -> ColorSet {
        if self.is_printed() {
            return printed;
        }
        printed.iter().fold(ColorSet::NONE, |set, c| set.with(self.color(c)))
    }

    /// The land or creature type printed as `printed`, as the text has it now: "other Goblins get +1/+1" says
    /// `subtype(Goblin)`.
    pub fn subtype(&self, printed: Subtype) -> Subtype {
        match self.word(Word::Subtype(printed)) {
            Word::Subtype(s) => s,
            Word::Color(_) => printed,
        }
    }

    /// Each subtype of `printed`, as the text has it now: a type line, or a token's types.
    pub fn subtypes(&self, printed: SubtypeSet) -> SubtypeSet {
        if self.is_printed() {
            return printed;
        }
        printed.iter().fold(SubtypeSet::NONE, |set, s| set.with(self.subtype(s)))
    }
}

impl fmt::Debug for SubtypeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Keyword abilities the core knows (CR 702), as a bitset. A keyword is here only when it changes a core procedure;
/// the rest are shared mechanics, built from the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Keyword {
    Flying = 0,
    Haste = 1,
    Vigilance = 2,
    Trample = 3,
    FirstStrike = 4,
    DoubleStrike = 5,
    Deathtouch = 6,
    Lifelink = 7,
    Reach = 8,
    Defender = 9,
    Flash = 10,
    Hexproof = 11,
    Indestructible = 12,
    Menace = 13,
    Infect = 14,
    Shadow = 15,
    Shroud = 16,
    /// "This spell can't be countered" (CR 101.2, 701.6a): a characteristic of the card.
    CantBeCountered = 17,
    /// "This creature can't block" (CR 509.1b).
    CantBlock = 18,
    /// "This creature can block only creatures with flying" (CR 509.1b).
    CanBlockOnlyCreaturesWithFlying = 19,
    /// "This permanent doesn't untap during your untap step" (CR 502.3).
    DoesNotUntap = 20,
    /// "This creature crews Vehicles as though its power were 2 greater" (CR 702.122b).
    CrewsAsThoughPowerTwoGreater = 21,
    /// "This creature can't be blocked" (CR 509.1b).
    CantBeBlocked = 22,
    /// "You can't spend mana to cast this spell" (CR 601.2h, 106.4): only what else pays a cost (convoke's taps,
    /// delve's exiles) can pay for it.
    CantSpendManaToCast = 23,
    /// "You may cast this card from your graveyard" (CR 113.6b, 601.3): for its ordinary cost.
    CastFromYourGraveyard = 24,
    /// Retrace (CR 702.81a): "you may cast this card from your graveyard by discarding a land card in addition to
    /// paying its other costs". Printed, or granted to cards in a graveyard by a static ability.
    Retrace = 25,
    /// Daybound (CR 702.145): it turns over with the day/night designation, and enters transformed at night.
    Daybound = 26,
    /// Nightbound (CR 702.145): the back face of a daybound card.
    Nightbound = 27,
    /// Split second (CR 702.61a): while a spell with it is on the stack, players can't cast spells or activate abilities
    /// that aren't mana abilities.
    SplitSecond = 28,

    /// "This permanent can't be attacked" (CR 508.1c): a planeswalker or battle that no creature may be declared
    /// attacking.
    CantBeAttacked = 29,
    /// "This creature can't be blocked by creatures with power 2 or less" (CR 509.1b): a blocker with no power can't
    /// block it either.
    CantBeBlockedByPowerTwoOrLess = 30,
    /// "This creature can't be blocked except by three or more creatures" (CR 509.1b).
    CantBeBlockedExceptByThreeOrMore = 31,
    /// Changeling (CR 702.73a): "This object is every creature type", a characteristic-defining ability that works in
    /// every zone (CR 604.3). The layers add the creature types (`cantrip-rules` `layers.rs`), so card code asks
    /// `has_subtype` as for any creature type. Printed on Firdoch Core.
    Changeling = 32,
    /// Decayed (CR 702.147a): "This creature can't block" and "When this creature attacks, sacrifice it at end of combat".
    /// A core keyword because a keyword counter gives it (CR 122.1b) in layer 6: the blocking rule reads it, and the
    /// layers give an object that has it once layer 6 is done the rules' triggered ability (`intrinsic::DECAYED`).
    Decayed = 33,
    /// "This creature can't be blocked by more than one creature" (CR 509.1b): an evasion ability, a restriction on the
    /// declaration of blockers. Printed (Stalking Tiger, Charging Rhino, Ironhoof Ox) or given (Meltstrider's Resolve,
    /// Familiar Ground, Challenger Troll, Vorrac Battlehorns).
    CantBeBlockedByMoreThanOne = 34,
    /// Islandwalk (CR 702.14): landwalk naming the land type Island, an evasion ability - "this creature can't be blocked as
    /// long as defending player controls an Island" (CR 702.14c). Printed or given (Lord of Atlantis's "other Merfolk ...
    /// have islandwalk"). Each basic landwalk is a keyword of its own that the blocking rule reads with its land type
    /// (`Keyword::LANDWALKS`), so a text change of the land type gives the landwalk it now names.
    Islandwalk = 35,
    /// Plainswalk (CR 702.14), as Islandwalk.
    Plainswalk = 36,
    /// Swampwalk (CR 702.14), as Islandwalk.
    Swampwalk = 37,
    /// Mountainwalk (CR 702.14), as Islandwalk.
    Mountainwalk = 38,
    /// Forestwalk (CR 702.14), as Islandwalk.
    Forestwalk = 39,
}

impl Keyword {
    /// Each basic landwalk keyword (CR 702.14a) and the basic land type it names.
    pub const LANDWALKS: [(Keyword, Subtype); 5] = [
        (Keyword::Plainswalk, Subtype::Plains),
        (Keyword::Islandwalk, Subtype::Island),
        (Keyword::Swampwalk, Subtype::Swamp),
        (Keyword::Mountainwalk, Subtype::Mountain),
        (Keyword::Forestwalk, Subtype::Forest),
    ];

    /// The landwalk naming `land_type` (CR 702.14a), if it is a basic land type: what "islandwalk" is once a text-changing
    /// effect has replaced Island with another basic land type (CR 612.2).
    pub fn landwalk(land_type: Subtype) -> Option<Keyword> {
        Keyword::LANDWALKS.iter().find(|&&(_, t)| t == land_type).map(|&(k, _)| k)
    }
}

/// Day and night (CR 731.1): the designation the game itself has, belonging to no player. A game starts with neither
/// and, once it has become day or night, has exactly one of them from then on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DayNight {
    #[default]
    Neither = 0,
    Day = 1,
    Night = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Keywords(pub u64);

impl Keywords {
    pub const NONE: Keywords = Keywords(0);

    pub const fn of(keywords: &[Keyword]) -> Keywords {
        let mut bits = 0u64;
        let mut i = 0;
        while i < keywords.len() {
            bits |= 1u64 << keywords[i] as u8;
            i += 1;
        }
        Keywords(bits)
    }

    pub fn has(self, k: Keyword) -> bool {
        self.0 & (1u64 << k as u8) != 0
    }

    pub fn with(self, k: Keyword) -> Keywords {
        Keywords(self.0 | (1u64 << k as u8))
    }
}

impl fmt::Debug for Keywords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keywords({:#x})", self.0)
    }
}

/// Counters an object can have (CR 122), by kind: one line each - the kind, its index in an object's dense counter array,
/// its name as Oracle text writes it (CR 122.1), and the token the scenario dump names it by (`Counters.PlusOnePlusOne`).
/// A kind a new card prints is one more line at the end: the host's view and events, proliferate, "remove all counters"
/// and the client's pills all walk `CounterKind::ALL` and name the kind, so nothing else changes.
macro_rules! counter_kinds {
    ($($kind:ident = $n:literal, $name:literal, $token:literal;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum CounterKind {
            $($kind = $n,)*
        }

        impl CounterKind {
            pub const ALL: [CounterKind; [$($n),*].len()] = [$(CounterKind::$kind),*];
            pub const COUNT: usize = Self::ALL.len();

            /// The counter's name as Oracle text writes it (CR 122.1).
            pub fn name(self) -> &'static str {
                match self {
                    $(CounterKind::$kind => $name,)*
                }
            }

            /// As the scenario dump names it: `Counters.PlusOnePlusOne`.
            pub fn token(self) -> &'static str {
                match self {
                    $(CounterKind::$kind => $token,)*
                }
            }
        }
    };
}

counter_kinds! {
    PlusOne = 0, "+1/+1", "PlusOnePlusOne";
    MinusOne = 1, "-1/-1", "MinusOneMinusOne";
    Loyalty = 2, "loyalty", "Loyalty";
    Time = 3, "time", "Time";
    Lore = 4, "lore", "Lore";
    Stun = 5, "stun", "Stun";
    Charge = 6, "charge", "Charge";
    Defense = 7, "defense", "Defense";
    Finality = 8, "finality", "Finality";
    Flying = 9, "flying", "Flying";
    Lifelink = 10, "lifelink", "Lifelink";
    Indestructible = 11, "indestructible", "Indestructible";
    Hexproof = 12, "hexproof", "Hexproof";
    Corpse = 13, "corpse", "Corpse";
    Fade = 14, "fade", "Fade";
    Level = 15, "level", "Level";
    Story = 16, "story", "Story";
    Wish = 17, "wish", "Wish";
    Rev = 18, "rev", "Rev";
    Mining = 19, "mining", "Mining";
    Influence = 20, "influence", "Influence";
    Oil = 21, "oil", "Oil";
    Shield = 22, "shield", "Shield";
    Fellowship = 23, "fellowship", "Fellowship";
    Hope = 24, "hope", "Hope";
    Void = 25, "void", "Void";
    Luck = 26, "luck", "Luck";
    Silver = 27, "silver", "Silver";
    Chorus = 28, "chorus", "Chorus";
    Burden = 29, "burden", "Burden";
    Incubation = 30, "incubation", "Incubation";
    Plan = 31, "plan", "Plan";
    // CR 122.1j: each one on an Equipment gives the creature it is attached to +1/+0 (the layers, 613.4c).
    Hone = 32, "hone", "Hone";
    Quest = 33, "quest", "Quest";
    // The rest of the keyword counters of CR 122.1b (flying, lifelink, indestructible and hexproof are above).
    FirstStrike = 34, "first strike", "FirstStrike";
    DoubleStrike = 35, "double strike", "DoubleStrike";
    Deathtouch = 36, "deathtouch", "Deathtouch";
    Decayed = 37, "decayed", "Decayed";
    Haste = 38, "haste", "Haste";
    Menace = 39, "menace", "Menace";
    Reach = 40, "reach", "Reach";
    Shadow = 41, "shadow", "Shadow";
    Trample = 42, "trample", "Trample";
    Vigilance = 43, "vigilance", "Vigilance";
    Page = 44, "page", "Page";
    Flood = 45, "flood", "Flood";
    Ingenuity = 46, "ingenuity", "Ingenuity";
    Net = 47, "net", "Net";
    Blight = 48, "blight", "Blight";
    Ice = 49, "ice", "Ice";
}

impl CounterKind {
    /// The keyword counters (CR 122.1b): "flying, first strike, double strike, deathtouch, decayed, exalted, haste, hexproof,
    /// indestructible, lifelink, menace, reach, shadow, trample, and vigilance". Exalted is not here: it is a triggered
    /// ability the core does not give (`triggers::EXALTED` is printed on the cards that have it), and no card in content puts
    /// an exalted counter.
    pub const KEYWORDS: [CounterKind; 14] = [
        CounterKind::Flying,
        CounterKind::FirstStrike,
        CounterKind::DoubleStrike,
        CounterKind::Deathtouch,
        CounterKind::Decayed,
        CounterKind::Haste,
        CounterKind::Hexproof,
        CounterKind::Indestructible,
        CounterKind::Lifelink,
        CounterKind::Menace,
        CounterKind::Reach,
        CounterKind::Shadow,
        CounterKind::Trample,
        CounterKind::Vigilance,
    ];

    /// The keyword a keyword counter grants (CR 122.1b).
    pub fn keyword(self) -> Option<Keyword> {
        match self {
            CounterKind::Flying => Some(Keyword::Flying),
            CounterKind::FirstStrike => Some(Keyword::FirstStrike),
            CounterKind::DoubleStrike => Some(Keyword::DoubleStrike),
            CounterKind::Deathtouch => Some(Keyword::Deathtouch),
            CounterKind::Decayed => Some(Keyword::Decayed),
            CounterKind::Haste => Some(Keyword::Haste),
            CounterKind::Hexproof => Some(Keyword::Hexproof),
            CounterKind::Indestructible => Some(Keyword::Indestructible),
            CounterKind::Lifelink => Some(Keyword::Lifelink),
            CounterKind::Menace => Some(Keyword::Menace),
            CounterKind::Reach => Some(Keyword::Reach),
            CounterKind::Shadow => Some(Keyword::Shadow),
            CounterKind::Trample => Some(Keyword::Trample),
            CounterKind::Vigilance => Some(Keyword::Vigilance),
            _ => None,
        }
    }
}

/// The steps of a turn (CR 500), in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Step {
    Untap = 0,
    Upkeep,
    Draw,
    PrecombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup,
}

impl Step {
    pub fn next(self) -> Option<Step> {
        use Step::*;
        Some(match self {
            Untap => Upkeep,
            Upkeep => Draw,
            Draw => PrecombatMain,
            PrecombatMain => BeginCombat,
            BeginCombat => DeclareAttackers,
            DeclareAttackers => DeclareBlockers,
            DeclareBlockers => CombatDamage,
            CombatDamage => EndOfCombat,
            EndOfCombat => PostcombatMain,
            PostcombatMain => End,
            End => Cleanup,
            Cleanup => return None,
        })
    }

    pub fn is_main(self) -> bool {
        matches!(self, Step::PrecombatMain | Step::PostcombatMain)
    }

    /// The C# engine's name for the step, which scenario waits and dumps use.
    pub fn name(self) -> &'static str {
        use Step::*;
        match self {
            Untap => "Untap",
            Upkeep => "Upkeep",
            Draw => "Draw",
            PrecombatMain => "PrecombatMain",
            BeginCombat => "BeginCombat",
            DeclareAttackers => "DeclareAttackers",
            DeclareBlockers => "DeclareBlockers",
            CombatDamage => "CombatDamage",
            EndOfCombat => "EndOfCombat",
            PostcombatMain => "PostcombatMain",
            End => "End",
            Cleanup => "Cleanup",
        }
    }
}

/// The kinds of zone (CR 400). Hand, library and graveyard are each player's own; the battlefield, the stack and
/// exile are shared, though exile is kept by owner for reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ZoneKind {
    Library = 0,
    Hand = 1,
    Graveyard = 2,
    Exile = 3,
    Battlefield = 4,
    Stack = 5,
    /// Not a zone: a token that has left the battlefield has ceased to exist (CR 111.7), and an object that has
    /// changed zones is a new object; the old row is kept here as last-known information.
    Gone = 6,
    /// The command zone (CR 114, 408): where emblems are, and nothing leaves it.
    Command = 7,
    /// Not a zone: the cards a player owns outside the game (CR 400.11), which in a match are their sideboard (CR 100.4,
    /// 400.11a, 108.3b). Nothing affects them but what brings them into the game (CR 400.11c), and only their owner may
    /// look at them.
    Sideboard = 8,
    /// Not a zone: the battlefield's phased-out permanents (CR 110.5, 702.26b), kept apart from the battlefield's list because
    /// a phased-out permanent is treated as though it does not exist. Phasing moves no object between zones (CR 702.26d): a
    /// permanent here is the same object it was, with its counters, status and attachments, and a token here goes on
    /// existing.
    PhasedOut = 9,
}

impl ZoneKind {
    pub fn is_hidden(self) -> bool {
        matches!(self, ZoneKind::Library | ZoneKind::Hand | ZoneKind::Sideboard)
    }

    pub fn name(self) -> &'static str {
        match self {
            ZoneKind::Library => "Library",
            ZoneKind::Hand => "Hand",
            ZoneKind::Graveyard => "Graveyard",
            ZoneKind::Exile => "Exile",
            ZoneKind::Battlefield => "Battlefield",
            ZoneKind::Stack => "Stack",
            ZoneKind::Gone => "Gone",
            ZoneKind::Command => "Command",
            ZoneKind::Sideboard => "Sideboard",
            ZoneKind::PhasedOut => "PhasedOut",
        }
    }
}

/// A set of zone kinds (CR 400.1): the zones a rule names, as "can't cast spells from graveyards or libraries" (CR 101.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ZoneKinds(pub u16);

impl ZoneKinds {
    pub const NONE: ZoneKinds = ZoneKinds(0);
    /// Every zone: "can't cast spells" wherever they are.
    pub const ALL: ZoneKinds = ZoneKinds(u16::MAX);
    /// The zones a spell can be cast from (CR 601.2a): every zone but the battlefield and the stack.
    pub const CAST_FROM: [ZoneKind; 5] = [ZoneKind::Hand, ZoneKind::Graveyard, ZoneKind::Exile, ZoneKind::Library, ZoneKind::Command];

    pub const fn of(kinds: &[ZoneKind]) -> ZoneKinds {
        let mut bits = 0u16;
        let mut i = 0;
        while i < kinds.len() {
            bits |= 1 << kinds[i] as u16;
            i += 1;
        }
        ZoneKinds(bits)
    }

    /// Every zone but these: "from anywhere other than their hands".
    pub const fn all_but(kinds: &[ZoneKind]) -> ZoneKinds {
        ZoneKinds(!ZoneKinds::of(kinds).0)
    }

    pub const fn contains(self, kind: ZoneKind) -> bool {
        self.0 & (1 << kind as u16) != 0
    }

    pub const fn union(self, other: ZoneKinds) -> ZoneKinds {
        ZoneKinds(self.0 | other.0)
    }

    pub const fn with(self, kind: ZoneKind) -> ZoneKinds {
        ZoneKinds(self.0 | (1 << kind as u16))
    }
}

/// A zone: its kind and, for the zones a player has, whose. The battlefield and the stack belong to nobody; their
/// `player` is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Zone {
    pub kind_raw: u8,
    pub player: PlayerId,
}

impl Zone {
    pub const fn new(kind: ZoneKind, player: PlayerId) -> Zone {
        Zone { kind_raw: kind as u8, player }
    }

    pub const BATTLEFIELD: Zone = Zone::new(ZoneKind::Battlefield, PlayerId(0));
    pub const STACK: Zone = Zone::new(ZoneKind::Stack, PlayerId(0));
    pub const GONE: Zone = Zone::new(ZoneKind::Gone, PlayerId(0));
    /// The command zone, shared: an emblem in it is its owner's by its `owner`.
    pub const COMMAND: Zone = Zone::new(ZoneKind::Command, PlayerId(0));
    /// The phased-out permanents, shared as the battlefield is (CR 702.26b): each is its controller's by its `controller`.
    pub const PHASED_OUT: Zone = Zone::new(ZoneKind::PhasedOut, PlayerId(0));

    pub fn library(p: PlayerId) -> Zone {
        Zone::new(ZoneKind::Library, p)
    }
    pub fn hand(p: PlayerId) -> Zone {
        Zone::new(ZoneKind::Hand, p)
    }
    pub fn graveyard(p: PlayerId) -> Zone {
        Zone::new(ZoneKind::Graveyard, p)
    }
    pub fn exile(p: PlayerId) -> Zone {
        Zone::new(ZoneKind::Exile, p)
    }
    /// The cards `p` owns outside the game (CR 400.11a): not a zone of the game, though the state keeps them as one.
    pub fn sideboard(p: PlayerId) -> Zone {
        Zone::new(ZoneKind::Sideboard, p)
    }

    pub fn kind(self) -> ZoneKind {
        match self.kind_raw {
            0 => ZoneKind::Library,
            1 => ZoneKind::Hand,
            2 => ZoneKind::Graveyard,
            3 => ZoneKind::Exile,
            4 => ZoneKind::Battlefield,
            5 => ZoneKind::Stack,
            7 => ZoneKind::Command,
            8 => ZoneKind::Sideboard,
            9 => ZoneKind::PhasedOut,
            _ => ZoneKind::Gone,
        }
    }

    pub fn is(self, kind: ZoneKind) -> bool {
        self.kind_raw == kind as u8
    }

    /// The zone's slot in the state's zone lists.
    pub fn slot(self, players: usize) -> usize {
        match self.kind() {
            ZoneKind::Battlefield => 4 * players,
            ZoneKind::Stack => 4 * players + 1,
            ZoneKind::Gone => 4 * players + 2,
            ZoneKind::Command => 4 * players + 3,
            ZoneKind::Sideboard => 4 * players + 4 + self.player.index(),
            ZoneKind::PhasedOut => 5 * players + 4,
            k => (k as usize) * players + self.player.index(),
        }
    }

    pub fn slots(players: usize) -> usize {
        5 * players + 5
    }
}

/// Why an object moved between zones, which triggers and replacements read (CR 700.4 "dies", 701.9 discard...).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZoneReason {
    Other,
    Draw,
    Discard,
    Mill,
    Sacrifice,
    Destroy,
    /// Destruction that "can't be regenerated" (CR 701.19c): regeneration shields do not replace it.
    DestroyNoRegen,
    PlayLand,
    Cast,
    Resolve,
    Counter,
    StateBased,
    Exile,
    Created,
    Search,
}

impl AbilityRef {
    /// Delayed abilities are numbered after a card's printed ones.
    pub const DELAYED: u8 = 128;
    /// Madness's linked triggered ability (CR 702.35a): the core's own, not an entry of any card's list.
    pub const MADNESS: u8 = 127;
    /// Rebound's delayed triggered ability (CR 702.88a): the core's own.
    pub const REBOUND: u8 = 126;
    /// Miracle's linked triggered ability (CR 702.94a): the core's own.
    pub const MIRACLE: u8 = 125;
    /// The history's marker of a granted way having been used (`cantrip_api::GRANTED_WAY_USED`).
    pub const GRANTED_WAY_USED: u8 = 124;
    /// The history's marker of extra loyalty activations allowed this turn (`actions::EXTRA_LOYALTY_ALLOWED`).
    pub const EXTRA_LOYALTY_ALLOWED: u8 = 123;
    /// A spell itself, in the history's record that it resolved (`Event::Resolved`, CR 608.2): no ability of the card.
    pub const SPELL: u8 = 122;

    pub fn spell(card: CardId) -> AbilityRef {
        AbilityRef { card, index: AbilityRef::SPELL }
    }

    pub fn is_spell(self) -> bool {
        self.index == AbilityRef::SPELL
    }

    pub fn miracle(card: CardId) -> AbilityRef {
        AbilityRef { card, index: AbilityRef::MIRACLE }
    }

    pub fn is_miracle(self) -> bool {
        self.index == AbilityRef::MIRACLE
    }

    pub fn madness(card: CardId) -> AbilityRef {
        AbilityRef { card, index: AbilityRef::MADNESS }
    }

    pub fn rebound(card: CardId) -> AbilityRef {
        AbilityRef { card, index: AbilityRef::REBOUND }
    }

    pub fn is_rebound(self) -> bool {
        self.index == AbilityRef::REBOUND
    }

    pub fn is_madness(self) -> bool {
        self.index == AbilityRef::MADNESS
    }

    pub fn delayed(card: CardId, index: u8) -> AbilityRef {
        AbilityRef { card, index: AbilityRef::DELAYED + index }
    }

    pub fn is_delayed(self) -> bool {
        self.index >= AbilityRef::DELAYED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CR 613.7, 613.8a(b): text changes chain whichever order they came in; a loop of two changes of one word applies the
    /// earlier.
    #[test]
    fn text_changes_of_a_word_apply_by_dependency() {
        let (elf, goblin, dragon) = (Word::Subtype(Subtype::Elf), Word::Subtype(Subtype::Goblin), Word::Subtype(Subtype::Dragon));
        assert_eq!(Words::NONE.changed(elf, goblin).changed(goblin, dragon).word(elf), dragon);
        assert_eq!(Words::NONE.changed(goblin, dragon).changed(elf, goblin).word(elf), dragon);
        assert_eq!(Words::NONE.changed(elf, goblin).changed(elf, dragon).word(elf), goblin);
        assert_eq!(Words::NONE.changed(elf, goblin).changed(goblin, elf).word(elf), elf);
    }

    /// CR 609.4b: paid as though mana were of any color, {1}{R}{R/G}{C}{X} is paid as {3}{C}{X} - the colored and hybrid
    /// symbols by any mana, the {C} by colorless only - and its mana value is unchanged.
    #[test]
    fn a_cost_paid_with_any_color_keeps_its_colorless_symbol() {
        let mut cost = ManaCost { generic: 1, x: 1, ..ManaCost::ZERO };
        cost.symbols[Mana::R as usize] = 1;
        cost.symbols[Mana::C as usize] = 1;
        cost.hybrid[0] = Some((Mana::R, Mana::G));
        let paid = cost.paid_with_any_color();
        assert_eq!(paid.generic, 3);
        assert_eq!(paid.symbols, [0, 0, 0, 0, 0, 1]);
        assert!(paid.hybrid.iter().all(Option::is_none));
        assert_eq!(paid.x, 1);
        assert_eq!(paid.mana_value(), cost.mana_value());
    }
}
