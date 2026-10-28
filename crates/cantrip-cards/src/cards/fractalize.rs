use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// X plus 1, as much as a power or toughness holds (an X of 32767 plus 1 is past an i16).
fn x_plus_one(x: u16) -> i16 {
    i16::try_from(u32::from(x) + 1).unwrap_or(i16::MAX)
}

/// "Until end of turn, target creature becomes a green and blue Fractal with base power and toughness each equal to X plus 1."
/// X as it was cast. "A green and blue Fractal" sets the creature type, replacing every other creature type and nothing else
/// (CR 205.1a, layer 4) and the colors, replacing every other (CR 105.3, layer 5); the base power and toughness are set in
/// layer 7b (CR 613.4b), so counters and pumps count on top. With its target gone the spell does not resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let words = g.words(item.object);
    let colors = ColorSet(ColorSet::of(Color::Green).0 | ColorSet::of(Color::Blue).0);
    let size = x_plus_one(item.x);
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::SetCreatureType(words.subtype(Subtype::Fractal)), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::SetColors(words.colors(colors)), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::SetPt(size, size), Duration::EndOfTurn);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::FRACTALIZE
};

#[cfg(test)]
mod tests {
    #[test]
    fn x_plus_one_does_not_wrap_past_the_width_of_a_size() {
        assert_eq!(super::x_plus_one(0), 1);
        assert_eq!(super::x_plus_one(32766), i16::MAX);
        assert_eq!(super::x_plus_one(32767), i16::MAX);
        assert_eq!(super::x_plus_one(u16::MAX), i16::MAX);
    }
}
