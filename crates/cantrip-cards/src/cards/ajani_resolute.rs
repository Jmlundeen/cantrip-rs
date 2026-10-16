use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Whenever you gain life" (CR 119.3, 603.2): one event for each time life is gained, and none when none is (a gain of 0,
/// or life that can't be gained, CR 119.7). Both Ajani and his Pridemate token trigger on it.
fn you_gain_life(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::LifeChange { player, delta } if *delta > 0 && *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "Put a loyalty counter on Ajani" (CR 122.1, 306.5b).
fn loyalty_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Loyalty, 1);
    Ok(())
}

/// 0: "You gain 1 life."
fn gain_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::gain_life(g, item, 1)
}

// --- The token ----------------------------------------------------------------------------------------------------

/// "Whenever you gain life, put a +1/+1 counter on this token."
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

/// The 2/2 white Cat Soldier creature token named Ajani's Pridemate (CR 111.4).
pub static PRIDEMATE: CardDef = CardDef {
    name: "Ajani's Pridemate",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Cat, Subtype::Soldier]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[triggered(
        when(EventMask::LIFE_CHANGE, you_gain_life),
        grow,
        "Whenever you gain life, put a +1/+1 counter on this token.",
    )],
    text: "Whenever you gain life, put a +1/+1 counter on this token.",
    ..CardDef::BLANK
};

/// \u{2212}4: "Create a 2/2 white Cat Soldier creature token named Ajani's Pridemate with ..."
fn make_pridemate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &PRIDEMATE, 1)?;
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

/// "Creatures you control get +2/+2" (layer 7c), from the command zone (CR 114.4).
fn creatures_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    view.state().obj(affected).controller == view.state().obj(source).controller && chars.is(CardType::Creature)
}

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    // CR 613.4b: a modifier only reaches something that has power and toughness to modify.
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

pub static EMBLEM: CardDef = CardDef {
    name: "Ajani Resolute emblem",
    has_cost: false,
    statics: &[static_ability(Layer::PtModify, creatures_you_control, plus_two_plus_two, "Creatures you control get +2/+2.")],
    text: "Creatures you control get +2/+2.",
    ..CardDef::BLANK
};

/// \u{2212}10: "You get an emblem with \"Creatures you control get +2/+2.\""
fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::LIFE_CHANGE, you_gain_life), loyalty_counter, "Whenever you gain life, put a loyalty counter on Ajani."),
        ability(loyalty(0), gain_one, "0: You gain 1 life."),
        ability(
            loyalty(-4),
            make_pridemate,
            "\u{2212}4: Create a 2/2 white Cat Soldier creature token named Ajani's Pridemate with \"Whenever you gain life, put a +1/+1 counter on this token.\"",
        ),
        ability(loyalty(-10), emblem, "\u{2212}10: You get an emblem with \"Creatures you control get +2/+2.\""),
    ],
    ..oracle::AJANI_RESOLUTE
};

pub static EXTRA: &[&CardDef] = &[&PRIDEMATE, &EMBLEM];
