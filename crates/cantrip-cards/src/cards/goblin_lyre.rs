use cantrip_api::*;
use cantrip_mechanics::{creatures_you_control, effects, targets};
use cantrip_oracle as oracle;

/// "Sacrifice this artifact: Flip a coin. If you win the flip, this artifact deals damage to target opponent or planeswalker equal to the number of creatures you control. If you lose the flip, this artifact deals damage to you equal to the number of creatures that opponent or that planeswalker's controller controls." (CR 608.2, 120.3).
fn flip_coin_if_win_flip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0).or_else(|| targets::object(g, item, 0).map(|object| g.controller(object))) else { return Ok(()) };
    let flip_0 = g.flip_coin(item.controller);
    g.end_sentence();
    if flip_0 {
        effects::damage_first_target_by_creatures_you_control(g, item)?;
        g.end_sentence();
    } else {
        let amount = creatures_you_control(g, opponent) as u32;
        g.damage(item.object, Target::Player(item.controller), amount);
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                SACRIFICE_THIS,
                flip_coin_if_win_flip,
                "Sacrifice this artifact: Flip a coin. If you win the flip, this artifact deals damage to target opponent or planeswalker equal to the number of creatures you control. If you lose the flip, this artifact deals damage to you equal to the number of creatures that opponent or that planeswalker's controller controls.",
            )
            .targets(&[targets::OPPONENT_OR_PLANESWALKER]),
    ],
    ..oracle::GOBLIN_LYRE
};
