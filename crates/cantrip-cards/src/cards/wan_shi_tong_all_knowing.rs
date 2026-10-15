use cantrip_api::*;
use cantrip_mechanics::{choose, targets, tokens, triggers::this_enters};
use cantrip_oracle as oracle;

/// "This token can't block or be blocked by non-Spirit creatures" (CR 509.1b): of any possible block it is in, the other
/// creature must be a Spirit.
fn spirit_blocking(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    let other = if source == blocker { attacker } else { blocker };
    q.chars(other).has_subtype(Subtype::Spirit)
}

/// The 1/1 colorless Spirit creature token with "This token can't block or be blocked by non-Spirit creatures."
pub static SPIRIT: CardDef = CardDef {
    name: "Spirit",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Spirit]),
    power: Some(1),
    toughness: Some(1),
    block_rules: &[spirit_blocking],
    shares_name: true,
    text: "This token can't block or be blocked by non-Spirit creatures.",
    ..CardDef::BLANK
};

/// Target nonland permanent's owner puts it into their library second from the top or on the bottom: their choice.
fn put_into_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(permanent).owner;
    let card = g.obj(item.object).card;
    let second_from_top = choose::one_of(g, owner, item.object, vec![Label::Card(card, 0), Label::Card(card, 1)])? == 0;
    g.put_into_library(&[permanent], if second_from_top { LibraryPlace::FromTop(1) } else { LibraryPlace::Bottom });
    Ok(())
}

/// "Whenever one or more cards are put into a library from anywhere" (CR 603.2c): cards, not tokens, and not moved within a
/// library. Judged as the permanent was (CR 603.10a). One trigger for each library, which is the batch's `data[1]`.
fn cards_into_a_library(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. } if to.is(ZoneKind::Library) && !from.is(ZoneKind::Library) && !q.obj(*old).token => {
            Some(Fired { subject: Target::Object(*new), data: [0, to.player.0 as u32] })
        }
        _ => None,
    }
}

fn make_spirits(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SPIRIT, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    labels: &["Second from the top", "On the bottom"],
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Wan Shi Tong enters, target nonland permanent's owner puts it into their library second from the top or on the bottom.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::NONLAND_PERMANENT],
            effect: put_into_library,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever one or more cards are put into a library from anywhere, create two 1/1 colorless Spirit creature tokens with \"This token can't block or be blocked by non-Spirit creatures.\"",
            listens: EventMask::ZONE_CHANGE,
            matches: cards_into_a_library,
            effect: make_spirits,
            look_back: true,
            once_per_batch: true,
            batch_by_data: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::WAN_SHI_TONG_ALL_KNOWING
};

pub static EXTRA: &[&CardDef] = &[&SPIRIT];
