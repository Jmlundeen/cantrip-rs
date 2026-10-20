use cantrip_api::*;
use cantrip_mechanics::{sacrifice_chosen, targets};
use cantrip_oracle as oracle;

fn is_creature(chars: &Chars) -> bool {
    chars.is(CardType::Creature)
}

fn is_enchantment(chars: &Chars) -> bool {
    chars.is(CardType::Enchantment)
}

fn is_planeswalker(chars: &Chars) -> bool {
    chars.is(CardType::Planeswalker)
}

/// "[The opponent] sacrifices a [kind] of their choice. If they can't, they lose 2 life and you draw a card." (CR 701.21a,
/// 608.2c): they can't when they control none of the kind or an effect stops the sacrifice (`sacrifice` sacrifices
/// nothing); they do not choose whether to sacrifice, only which permanent.
fn process(g: &mut dyn Game, item: &StackItem, opponent: PlayerId, kind: fn(&Chars) -> bool) -> Result<(), Suspend> {
    if sacrifice_chosen(g, opponent, item.object, 1, kind)?.is_empty() {
        g.lose_life(opponent, 2);
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// "Target opponent sacrifices a creature of their choice. If they can't, they lose 2 life and you draw a card. Then
/// repeat this process for an enchantment and a planeswalker." The same player each time, whether or not an earlier
/// sacrifice was possible.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    process(g, item, opponent, is_creature)?;
    process(g, item, opponent, is_enchantment)?;
    process(g, item, opponent, is_planeswalker)
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], effect), ..oracle::INVOKE_DESPAIR };
