use cantrip_api::*;
use cantrip_mechanics::{exile_target, library};
use cantrip_oracle as oracle;

/// "Permanent that's one or more colors": checked as it is chosen and again as the ability resolves (CR 115.1, 608.2b).
fn colored_permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if is_live(q, id) && q.is_on_battlefield(id) && !q.chars(id).colors.is_empty())
}

/// "Exile up to one target permanent that's one or more colors."
fn exile_colored(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count > 0 {
        exile_target(g, item, 0);
    }
    Ok(())
}

/// "Whenever you cast a colorless spell": judged on the spell as it is on the stack, not by its mana cost.
fn colorless_spell_you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && q.chars(*spell).colors.is_empty() => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// +2: You gain 3 life and draw a card.
fn plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    g.draw(item.controller, 1);
    Ok(())
}

/// 0: Add {C}{C}{C}.
fn zero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::C, 3);
    Ok(())
}

/// -11: Search your library for any number of colorless nonland cards, exile them, then shuffle. Until end of turn, you
/// may cast those cards without paying their mana costs.
fn minus_eleven(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let library = g.state().zone(Zone::library(player)).len() as u8;
    let exiled = library::search_to_exile(g, player, player, item.object, library::Find::ONE.up_to(library), &[], |q, c| {
        let chars = q.chars(c);
        !chars.is(CardType::Land) && chars.colors.is_empty()
    })?;
    for card in exiled {
        if g.obj(card).zone.is(ZoneKind::Exile) {
            g.allow_play(
                card,
                player,
                PlayGrant { instead: Some(ManaCost::ZERO), expiry: Expiry::EndOfTurn, ..PlayGrant::CAST },
            );
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When you cast this spell, exile up to one target permanent that's one or more colors.",
            listens: EventMask::SPELL_CAST,
            matches: cantrip_mechanics::triggers::cast_this_spell,
            targets: &[TargetSpec { min: 0, max: 1, filter: colored_permanent }],
            effect: exile_colored,
            anywhere: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you cast a colorless spell, exile up to one target permanent that's one or more colors.",
            listens: EventMask::SPELL_CAST,
            matches: colorless_spell_you_cast,
            targets: &[TargetSpec { min: 0, max: 1, filter: colored_permanent }],
            effect: exile_colored,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "+2: You gain 3 life and draw a card.",
            cost: Cost { loyalty: Some(2), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: plus_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "0: Add {C}{C}{C}.",
            cost: Cost { loyalty: Some(0), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: zero,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}11: Search your library for any number of colorless nonland cards, exile them, then shuffle. Until end of turn, you may cast those cards without paying their mana costs.",
            cost: Cost { loyalty: Some(-11), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: minus_eleven,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::UGIN_EYE_OF_THE_STORMS
};
