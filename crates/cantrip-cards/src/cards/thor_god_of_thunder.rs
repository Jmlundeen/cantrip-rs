use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target Equipment, instant, or sorcery card from your graveyard."
fn equipment_instant_or_sorcery_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[]) && matches!(t, Target::Object(id) if {
        let c = q.chars(id);
        c.is(CardType::Instant) || c.is(CardType::Sorcery) || c.has_subtype(Subtype::Equipment)
    })
}

/// Exile the target card; until the end of its controller's next turn they may play it (CR 601.3, 305.1). The
/// permission is for the card that actually reaches exile (CR 400.7): a replacement that sent it elsewhere leaves
/// nothing to play.
fn exile_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let exiled = g.exile(card);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(
            exiled,
            item.controller,
            PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST },
        );
    }
    Ok(())
}

/// "Whenever you cast a noncreature spell": the ability is about the spell.
fn you_cast_a_noncreature_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) && !q.chars(*spell).is(CardType::Creature) => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

/// "Thor deals damage equal to that spell's mana value to any target": the spell's as the ability resolves, or as it last
/// was on the stack if it has left (CR 608.2h) - an {X} in its cost counting as the X announced (CR 202.3e).
fn spell_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let amount = g.mana_value(spell);
    if amount > 0 {
        // A mana value past what damage holds (RX-5) deals that limit, never a wrapped amount.
        damage_target(g, item, 0, amount.min(u16::MAX as u32) as u16);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_and_play, "When Thor enters, exile target Equipment, instant, or sorcery card from your graveyard. Until the end of your next turn, you may play that card.")
            .targets(&[target(equipment_instant_or_sorcery_in_your_graveyard)]),
        triggered(when(EventMask::SPELL_CAST, you_cast_a_noncreature_spell), spell_damage, "Whenever you cast a noncreature spell, Thor deals damage equal to that spell's mana value to any target.")
            .targets(&[targets::ANY]),
    ],
    ..oracle::THOR_GOD_OF_THUNDER
};
