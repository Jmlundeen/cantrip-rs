use cantrip_api::*;
use cantrip_mechanics::{airbend, triggers::THIS_ENTERS, targets, tokens};
use cantrip_oracle as oracle;

/// The 1/1 white Ally creature token it makes.
pub static ALLY: CardDef = CardDef {
    name: "Ally",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Ally]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "Any number of other target nonland permanents you control."
fn other_nonland_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && q.controller(id) == asker.controller)
        && (targets::NONLAND_PERMANENT.filter)(q, asker, t)
}

fn airbend_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    airbend(g, item);
    Ok(())
}

/// "Whenever you cast a spell from exile": where a spell was cast from is recorded on it (CR 601.2a).
fn you_cast_from_exile(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source) && q.obj(*spell).cast_from == Some(ZoneKind::Exile) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn ally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ALLY, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, airbend_them, "When Appa enters, airbend any number of other target nonland permanents you control.")
            .targets(&[TargetSpec { min: 0, max: 16, filter: other_nonland_permanent_you_control, label: "" }]),
        triggered(when(EventMask::SPELL_CAST, you_cast_from_exile), ally, "Whenever you cast a spell from exile, create a 1/1 white Ally creature token."),
    ],
    ..oracle::APPA_STEADFAST_GUARDIAN
};

pub static EXTRA: &[&CardDef] = &[&ALLY];
