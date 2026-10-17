use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, craft};
use cantrip_oracle as oracle;

/// "You may sacrifice a nontoken creature. If you do, draw X cards, then mill X cards, where X is that creature's power."
/// Declining is the empty answer to an up-to-one choice; X is the creature's power as it last existed on the battlefield
/// (CR 608.2h), none for a negative one.
fn sacrifice_to_draw_and_mill(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> =
        g.permanents_of(you).into_iter().filter(|&id| !g.obj(id).token && g.chars(id).is(CardType::Creature)).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Sacrifice, "nontoken creature", creatures.clone(), creatures, 1, true)?;
    let Some(&creature) = chosen.first() else { return Ok(()) };
    // A permanent of your own is always yours to sacrifice to your own ability (CR 701.21a): chosen, it is sacrificed.
    g.sacrifice(&[creature]);
    let x = g.chars(creature).power.unwrap_or(0).max(0) as u32;
    g.draw(you, x);
    g.mill(you, x);
    Ok(())
}

/// The keywords the Bonemass has while an exiled card used to craft it has them.
const SHARED: [Keyword; 12] = [
    Keyword::Flying,
    Keyword::FirstStrike,
    Keyword::DoubleStrike,
    Keyword::Deathtouch,
    Keyword::Haste,
    Keyword::Hexproof,
    Keyword::Indestructible,
    Keyword::Lifelink,
    Keyword::Menace,
    Keyword::Reach,
    Keyword::Trample,
    Keyword::Vigilance,
];

/// "This creature has flying as long as an exiled card used to craft it has flying. The same is true for first strike, double
/// strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, protection, reach, trample, and vigilance." (CR
/// 613.1f) Each of these the exiled cards have, as they are in exile, it has; protection with whatever it is from (CR
/// 702.16a).
fn crafted_keywords(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    for card in craft::used_to_craft(q.state(), source) {
        let c = q.chars(card);
        for k in SHARED {
            if c.keywords.has(k) {
                chars.keywords = chars.keywords.with(k);
            }
        }
        chars.protection = ColorSet(chars.protection.0 | c.protection.0);
        chars.protection_colorless |= c.protection_colorless;
        chars.protection_players |= c.protection_players;
        chars.protection_types = TypeSet(chars.protection_types.0 | c.protection_types.0);
        chars.protection_colored_spells |= c.protection_colored_spells;
        chars.protection_everything |= c.protection_everything;
    }
}

pub static BACK: CardDef = CardDef {
    statics: &[
        characteristic_defining(Layer::PtDefine, craft::power_and_toughness_of_total_crafted_power, "Wretched Bonemass's power and toughness are each equal to the total power of the exiled cards used to craft it.").lost_pt(0, 0),
        static_ability(Layer::Ability, cantrip_mechanics::statics::itself, crafted_keywords, "This creature has flying as long as an exiled card used to craft it has flying. The same is true for first strike, double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, protection, reach, trample, and vigilance."),
    ],
    ..oracle::WRETCHED_BONEMASS
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, sacrifice_to_draw_and_mill, "When this artifact enters, you may sacrifice a nontoken creature. If you do, draw X cards, then mill X cards, where X is that creature's power."),
        craft::ability(mana("{2}{B}{B}"), &craft::ONE_OR_MORE_CREATURES, "Craft with one or more creatures {2}{B}{B}"),
        ability(cost("{2}{B}"), cantrip_mechanics::effects::return_this_to_hand,"{2}{B}: Return this card from your graveyard to your hand.").from(ZoneKind::Graveyard),
    ],
    back: Some(&BACK),
    ..oracle::ALTAR_OF_THE_WRETCHED
};
