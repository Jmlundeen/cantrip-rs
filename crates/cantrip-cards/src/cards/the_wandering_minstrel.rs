use cantrip_api::*;
use cantrip_mechanics::{effects, replacements, tokens, triggers};
use cantrip_oracle as oracle;

/// The 2/2 Elemental creature token that's all colors (CR 105.2).
pub static ELEMENTAL: CardDef = CardDef {
    colors: Some(ColorSet(0b11111)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..token!(Elemental)
};

fn towns(q: &dyn Query, player: PlayerId) -> usize {
    q.permanents_of(player).into_iter().filter(|&id| q.chars(id).has_subtype(Subtype::Town)).count()
}

/// "if you control five or more Towns" (CR 603.4): an intervening if, asked as it triggers and again as it resolves.
fn five_or_more_towns(q: &dyn Query, asker: &Asker) -> bool {
    towns(q, asker.controller) >= 5
}

fn make_elemental(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ELEMENTAL, 1)?;
    Ok(())
}

/// "Other creatures you control get +X/+X until end of turn, where X is the number of Towns you control": X is
/// counted once, as the ability resolves (CR 608.2h, the ruling).
fn pump_by_towns(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = towns(g, item.controller) as i16;
    effects::pump_other_creatures_you_control(g, item, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacements::LANDS_YOU_CONTROL_ENTER_UNTAPPED],
    abilities: &[
        triggered(
            when_asked(EventMask::STEP_BEGAN, triggers::beginning_of_your_combat),
            make_elemental,
            "The Minstrel's Ballad — At the beginning of combat on your turn, if you control five or more Towns, create a 2/2 Elemental creature token that's all colors.",
        )
        .condition(five_or_more_towns),
        ability(
            cost("{3}{W}{U}{B}{R}{G}"),
            pump_by_towns,
            "{3}{W}{U}{B}{R}{G}: Other creatures you control get +X/+X until end of turn, where X is the number of Towns you control.",
        ),
    ],
    ..oracle::THE_WANDERING_MINSTREL
};

pub static EXTRA: &[&CardDef] = &[&ELEMENTAL];
