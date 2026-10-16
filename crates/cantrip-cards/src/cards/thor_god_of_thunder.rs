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

/// "Whenever you cast a noncreature spell": the spell's mana value is read as it is cast, X included (CR 202.3e), so
/// the ability deals it even if the spell has left the stack by then (CR 608.2h).
fn you_cast_a_noncreature_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) => {
            let chars = q.chars(*spell);
            if chars.is(CardType::Creature) {
                return None;
            }
            let cost = q.card(chars.card).cost;
            let x = q.state().stack.iter().find(|i| i.object == *spell).map_or(0, |i| i.x as u32);
            Some(Fired { subject: Target::Object(*spell), data: [cost.mana_value() + cost.x as u32 * x, 0] })
        }
        _ => None,
    }
}

fn spell_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.data[0] > 0 {
        damage_target(g, item, 0, item.data[0] as u16);
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
