use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature can't block this turn. If this spell was kicked, gain control of that creature until end of turn, untap it,
/// and it gains haste until end of turn. Create a Clue token." (CR 509.1b, 702.33, 613.1b): the one target illegal on
/// resolution, the spell does nothing at all (CR 608.2b), so no Clue is made. Creating a Clue is not investigating (the
/// card's ruling).
fn brainwashing(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::first_target_cant_block_this_turn(g, item)?;
    g.end_sentence();
    if item.kicked > 0 {
        effects::gain_control_untap_haste(g, item);
        g.end_sentence();
    }
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: kicker(mana("{3}")),
    spell: spell(&[targets::CREATURE], brainwashing),
    ..oracle::JETS_BRAINWASHING
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
