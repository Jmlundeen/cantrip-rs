//! Land templates: the lands of a printed cycle differ in their colors and little else (CR 305). A land's file names its
//! colors and uses these. The colors are numbered as `Color` and `Mana` number them (`W`..`G` below), the basic land
//! types as `Subtype`'s discriminant (`sub(Subtype::Plains)`), because a function pointer cannot capture them.
//!
//! The "enters tapped" part of a land is `mana::enters_tapped`, `mana::enters_tapped_unless_pay_two_life` (shock lands) or
//! `enters_tapped_unless_you_control` or `enters_tapped_unless_you_control_either` below; the card sets it in `enters`.

use crate::library::{search_to_battlefield, search_to_hand, Find};
use crate::{choose, effects, mana, triggers};
use cantrip_api::*;

/// A subtype as the number the generic land templates take: `sub(Subtype::Island)`.
pub const fn sub(subtype: Subtype) -> u16 {
    subtype as u16
}

/// The subtype a template's number names, as `source`'s text has that word now (CR 612.2).
pub fn word(q: &dyn Query, source: ObjectId, n: u16) -> Subtype {
    let printed = Subtype::ALL.into_iter().find(|&s| s as u16 == n).expect("a subtype's number");
    q.words(source).subtype(printed)
}

/// Whether `controller` controls a permanent with the land type `land_type`, through the layers (CR 305.6, 613). Arena of
/// Glory, Castle Locthwain and Shifting Woodland's "unless you control a [land type]" and Cori Mountain Monastery's "unless
/// you control a Plains or an Island" ask it.
pub fn controls_land_type(q: &dyn Query, controller: PlayerId, land_type: Subtype) -> bool {
    q.state().battlefield().iter().any(|&id| q.controller(id) == controller && q.chars(id).has_subtype(land_type))
}

/// "This land enters tapped. As it enters, choose a color." (CR 614.1c, 614.12a): both as it enters, the color kept on the
/// permanent (CR 607.2d). Sunken Citadel and Valgavoth's Lair print it.
pub fn enters_tapped_choosing_a_color(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    entering.chosen_color = Some(crate::choose_color_for(g, entering.controller, entering.object, ColorUse::Benefit)?);
    Ok(())
}

/// "Mana of the chosen color" (CR 106.5): the one color chosen as it entered, none if none was (a land that became a copy of it
/// without entering), and then the ability adds no mana. "The chosen color" is no color word, so a text-changing effect leaves it
/// as it is (CR 612.2). The colors for a mana ability's `colors_from`: Sunken Citadel, Valgavoth's Lair and Heraldic Banner.
pub fn chosen_color(q: &dyn Query, asker: &Asker) -> ColorSet {
    q.obj(asker.source).chosen_color.map_or(ColorSet::NONE, ColorSet::of)
}

/// "This land enters tapped unless you control a [land type]" (CR 614.1c): read on the battlefield as it is while the land
/// enters, so the land itself is not counted, through the layers (a land of any kind may have the type). Arena of Glory
/// (Mountain), Castle Locthwain (Swamp) and Shifting Woodland (Forest) print it.
pub fn enters_tapped_unless_you_control<const TYPE: u16>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let land_type = word(g, entering.object, TYPE);
    if !controls_land_type(g, entering.controller, land_type) {
        entering.tapped = true;
    }
    Ok(())
}

/// "This land enters tapped unless you control a basic land" (CR 614.1c, 205.4a): the supertype, not a land type, read
/// on the battlefield as it is while the land enters, so the land itself is not counted. Abandoned Air Temple and Agna
/// Qel'a print it.
pub fn enters_tapped_unless_you_control_basic_land(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let controls_basic = g
        .permanents_of(entering.controller)
        .into_iter()
        .any(|id| g.obj(id).became.is_none() && g.chars(id).is(CardType::Land) && g.chars(id).supertypes.basic);
    if !controls_basic {
        entering.tapped = true;
    }
    Ok(())
}

/// "This land enters tapped unless you control a [land type] or a [land type]" (CR 614.1c): as `enters_tapped_unless_you_control`
/// with two land types, each read through the layers and the land's words (CR 612.2). Cori Mountain Monastery (Plains or
/// Island) and Mistrise Village (Mountain or Forest) print it.
pub fn enters_tapped_unless_you_control_either<const A: u16, const B: u16>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let (a, b) = (word(g, entering.object, A), word(g, entering.object, B));
    if !controls_land_type(g, entering.controller, a) && !controls_land_type(g, entering.controller, b) {
        entering.tapped = true;
    }
    Ok(())
}

/// "Destroy target [permanent]. Its controller may search their library for a basic land card, put it onto the battlefield
/// [tapped], then shuffle": the permanent `item` targeted in `slot`, if it is still a legal target (CR 608.2b). Its controller
/// is the one it last had (CR 608.2h), asked to search after the destruction and even when the permanent was not destroyed
/// (indestructible, regenerated); declining skips the search and the shuffle. Erode (tapped), Ghost Quarter (untapped) and
/// Avengers Disassembled (tapped) print it.
pub fn destroy_target_its_controller_may_fetch_basic(g: &mut dyn Game, item: &StackItem, slot: usize, tapped: bool) -> Result<(), Suspend> {
    let Some(permanent) = crate::targets::object(g, item, slot) else { return Ok(()) };
    let owner = g.controller(permanent);
    g.destroy(permanent);
    may_fetch_basic(g, owner, item.object, tapped)
}

/// "[Player] may search their library for a basic land card, put it onto the battlefield [tapped], then shuffle": asked
/// first whether to, and declining skips the search and the shuffle (CR 701.23). Ghost Quarter, Erode and Avengers
/// Disassembled (through `destroy_target_its_controller_may_fetch_basic`) and Demolition Field's own search print it.
pub fn may_fetch_basic(g: &mut dyn Game, player: PlayerId, source: ObjectId, tapped: bool) -> Result<(), Suspend> {
    if !effects::ask_may(g, player, source, Purpose::Ask(if tapped { &FETCH_BASIC_TAPPED } else { &FETCH_BASIC }))? {
        return Ok(());
    }
    search_basic_land_to_battlefield(g, player, source, tapped)
}

/// "Search your library for a basic land card, put it onto the battlefield [tapped], then shuffle" (CR 701.23): the search
/// of a Ghost Quarter, a Prismatic Vista and a Lander token (Biotech Specialist), the player finding one or none.
pub fn search_basic_land_to_battlefield(g: &mut dyn Game, player: PlayerId, source: ObjectId, tapped: bool) -> Result<(), Suspend> {
    fetch_basic_land_to_battlefield(g, player, source, tapped)?;
    Ok(())
}

/// `search_basic_land_to_battlefield`, giving the land that arrived (none if the player found none) for an ability that goes
/// on to do something to it: Elven Passage's "untap that land".
pub fn fetch_basic_land_to_battlefield(g: &mut dyn Game, player: PlayerId, source: ObjectId, tapped: bool) -> Result<Vec<ObjectId>, Suspend> {
    search_to_battlefield(g, player, source, Find::ONE, tapped, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })
}

/// What "you may search your library for a basic land card, reveal it, put it into your hand, then shuffle" asks.
pub const FETCH_BASIC_TO_HAND: Question = Question::new(
    "SearchLibrary",
    "search your library for a basic land card?",
    "Reveal it and put it into your hand, then shuffle",
    "Search",
    "Don't search",
);

/// "Search your library for a basic land card, reveal it, put it into your hand, then shuffle" (CR 701.23): the player
/// finding one or none, and the library shuffled either way. Bushwhack's first mode and Environmental Scientist print it.
pub fn search_basic_land_to_hand(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    search_to_hand(g, player, source, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

/// "You may search your library for a basic land card, reveal it, put it into your hand, then shuffle": asked first
/// whether to, and declining skips the search and the shuffle (CR 701.23). Environmental Scientist's enters trigger
/// (`search_basic_land_to_hand` is Bushwhack's, which does not say "may").
pub fn may_search_basic_land_to_hand(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    if !effects::ask_may(g, player, source, Purpose::Ask(&FETCH_BASIC_TO_HAND))? {
        return Ok(());
    }
    search_basic_land_to_hand(g, player, source)
}

fn return_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let lands: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).collect();
    if let Some(land) = choose::card(g, player, item.object, Purpose::ReturnToHand, lands)? {
        g.return_to_hand(&[land]);
    }
    Ok(())
}

/// The bounce lands' (karoos') two abilities, mana ability first: "{T}: Add {A}{B}." (`text` is its full text) and "When
/// this land enters, return a land you control to its owner's hand" (CR 603.6a): chosen as the ability resolves, not
/// targeted, and the land itself may be the one; asked whenever there is a land to choose. With `enters:
/// Some(mana::enters_tapped)` it is Azorius Chancery, Orzhov Basilica and the rest of the cycle.
pub const fn bounce_land(a: Mana, b: Mana, text: &'static str) -> [AbilityDef; 2] {
    [
        mana_ability(TAP, add_each(&[a, b]), text),
        triggered(triggers::THIS_ENTERS, return_a_land, "When this land enters, return a land you control to its owner's hand."),
    ]
}

fn fetch_two_types<const A: u16, const B2: u16>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (a, b) = (word(g, item.object, A), word(g, item.object, B2));
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, move |q, c| {
        let chars = q.chars(c);
        chars.has_subtype(a) || chars.has_subtype(b)
    })?;
    Ok(())
}

/// A fetch land's ability (CR 701.23): "{T}, Pay 1 life, Sacrifice this land: Search your library for a [type A] or [type B]
/// card, put it onto the battlefield, then shuffle." `text` is the full text. The types are the ones the land's text named
/// as it was sacrificed (CR 612.2, 113.7a). Flooded Strand and Arid Mesa print it.
pub const fn fetch_land<const A: u16, const B2: u16>(text: &'static str) -> AbilityDef {
    ability(TAP.and(life(1)).and(SACRIFICE_THIS), fetch_two_types::<A, B2>, text)
}

fn fetch_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_basic_land_to_battlefield(g, item.controller, item.object, false)
}

/// A fetch land that finds any basic land: "{T}, Pay 1 life, Sacrifice this land: Search your library for a basic land
/// card, put it onto the battlefield, then shuffle." Prismatic Vista prints it, and so does every land that borrows its
/// text.
pub const fn fetch_basic_land(text: &'static str) -> AbilityDef {
    ability(TAP.and(SACRIFICE_THIS).and(life(1)), fetch_basic, text)
}

/// The triomes' "Cycling {3}" (CR 702.29): with `enters: Some(mana::enters_tapped)` and its three basic land types, a
/// triome. Raffine's Tower and Savai Triome print it.
pub const TRIOME_CYCLING: AbilityDef = triggers::cycling(mana("{3}"), "Cycling {3} ({3}, Discard this card: Draw a card.)");

/// The surveil lands' "When this land enters, surveil 1" (CR 701.25): with `enters: Some(mana::enters_tapped)`, Meticulous
/// Archive and Undercity Sewers.
pub const SURVEIL_LAND: AbilityDef = triggered(triggers::THIS_ENTERS, |g, item| effects::surveil(g, item, 1), "When this land enters, surveil 1.");

/// A horizon land's mana ability: "{T}, Pay 1 life: Add {A} or {B}." (`text` is its full text), the color named as it is
/// activated (CR 106.5). Horizon Canopy and Fiery Islet print it.
pub const fn pay_life_for_either(a: Color, b: Color, text: &'static str) -> AbilityDef {
    mana_ability(TAP.and(life(1)), add_any_color(1).colors(ColorSet(ColorSet::of(a).0 | ColorSet::of(b).0)), text)
}

/// A horizon land's other ability: "{1}, {T}, Sacrifice this land: Draw a card." (`text` is its full text, with the land's
/// name). Horizon Canopy, Fiery Islet and Cryptic Caves (with its own restriction) print it.
pub const fn sacrifice_to_draw(text: &'static str) -> AbilityDef {
    ability(cost("{1}, {T}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), text)
}

fn controls_either<const A: u16, const B2: u16>(q: &dyn Query, asker: &Asker) -> bool {
    controls_land_type(q, asker.controller, word(q, asker.source, A)) || controls_land_type(q, asker.controller, word(q, asker.source, B2))
}

/// A verge's two mana abilities (CR 602.5b): "{T}: Add {PRIMARY}." and "{T}: Add {SECOND}. Activate only if you control a
/// [type A] or a [type B]", read through the layers, the types as the verge's text names them (CR 612.2). Blazemire Verge and Thornspire Verge print it.
pub const fn verge<const A: u16, const B2: u16>(
    primary: Mana,
    second: Mana,
    primary_text: &'static str,
    second_text: &'static str,
) -> [AbilityDef; 2] {
    [
        mana_ability(TAP, add(primary), primary_text),
        mana_ability(TAP, add(second), second_text).condition(controls_either::<A, B2>),
    ]
}

/// A manland's animation (CR 613.1d, 613.4b): "Until end of turn, this land becomes a P/T [colors] [subtype] creature
/// [with keywords]. It's still a land." The colors and the creature type are the ones the land's own text has now (CR 612),
/// read through its words; the base power and toughness, the keywords and the type are laid over the land (see
/// `animate`). Restless Anchorage, Restless Reef, Restless Vinestalk, Creeping Tar Pit and Celestial Colonnade print it.
pub fn becomes_creature_until_end_of_turn(
    g: &mut dyn Game,
    item: &StackItem,
    (power, toughness): (i16, i16),
    colors: ColorSet,
    subtype: Subtype,
    keywords: Keywords,
) {
    let words = g.words(item.object);
    let animation = crate::Animation {
        power,
        toughness,
        colors: Some(words.colors(colors)),
        subtype: Some(words.subtype(subtype)),
        all_creature_types: false,
        keywords,
        ability: None,
    };
    crate::animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
}

/// What "[player] may search their library for a basic land card, put it onto the battlefield, then shuffle" asks.
pub const FETCH_BASIC: Question = Question::new(
    "SearchLibrary",
    "search your library for a basic land card?",
    "It goes onto the battlefield, then you shuffle",
    "Search",
    "Don't search",
);

/// "Activate only if this land entered this turn or if you control a basic land" (CR 602.5b): checked as the ability is
/// activated. Entering is asked of the event history (CR 400.7: a land that left and came back is a new object that entered
/// this turn); a basic land is a land with the basic supertype (CR 205.4a), read through the layers. The land itself is not
/// basic, so it never counts itself. Gleaming Bastion, Training Compound and Hidden Lair print it.
pub fn entered_this_turn_or_you_control_a_basic_land(q: &dyn Query, asker: &Asker) -> bool {
    crate::history::entered_this_turn(q.history(), asker.source)
        || q.permanents_of(asker.controller).into_iter().any(|id| {
            let chars = q.chars(id);
            chars.is(CardType::Land) && chars.supertypes.basic
        })
}

/// The same search, the land put onto the battlefield tapped (Path to Exile, Erode).
pub const FETCH_BASIC_TAPPED: Question = Question::new(
    "SearchLibrary",
    "search your library for a basic land card?",
    "It enters tapped, then you shuffle",
    "Search",
    "Don't search",
);

/// "The number of [lands] you control", the value of a characteristic-defining ability (CR 604.3, 613.1): the permanents on the
/// battlefield `counts` accepts, read through the layers before the one being applied (so a land made an Island in layer 4 is
/// counted by a layer 7a ability). "You" is the controller of `source` (CR 108.4: the engine keeps the owner as controller off the
/// battlefield and stack, CR 108.4a, and the caster on the stack): Lumbering Worldwagon's power and Sandman's power and toughness ("works in all zones")
/// count lands; `counts` is `|c| c.is(CardType::Land)` for them.
pub fn count_you_control(q: &dyn LayerQuery, source: ObjectId, counts: &dyn Fn(&Chars) -> bool) -> i16 {
    let state = q.state();
    let obj = state.obj(source);
    state.battlefield().iter().filter(|&&id| state.obj(id).controller == obj.controller && counts(&q.chars(id))).count() as i16
}
