use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Whenever an opponent casts a creature spell, investigate." (CR 603.2, 701.40a): any opponent's creature spell,
/// read as it stands on the stack. Investigating creates a Clue token for the ability's controller.
fn when_an_opponent_casts_a_creature_spell(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if q.state().opponents(asker.controller).any(|o| o == *player) && q.chars(*spell).is(CardType::Creature) {
        return Fired::PLAIN;
    }
    None
}

/// "Investigate." (CR 701.40a): create a Clue token, an artifact with "{2}, Sacrifice this token: Draw a card."
fn investigate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::investigate(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, when_an_opponent_casts_a_creature_spell),
        investigate,
        "Whenever an opponent casts a creature spell, investigate. (Create a Clue token. It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
    )],
    ..oracle::SAM_KIRK_XENOANTHROPOLOGIST
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
