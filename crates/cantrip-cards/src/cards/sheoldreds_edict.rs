use cantrip_api::*;
use cantrip_mechanics::each_opponent_sacrifices;
use cantrip_oracle as oracle;

fn nontoken_creature(g: &dyn Query, id: ObjectId) -> bool {
    g.chars(id).is(CardType::Creature) && !g.obj(id).token
}

fn creature_token(g: &dyn Query, id: ObjectId) -> bool {
    g.chars(id).is(CardType::Creature) && g.obj(id).token
}

fn planeswalker(g: &dyn Query, id: ObjectId) -> bool {
    g.chars(id).is(CardType::Planeswalker)
}

fn sacrifice_nontoken(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, nontoken_creature)
}

fn sacrifice_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, creature_token)
}

fn sacrifice_planeswalker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, planeswalker)
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Each opponent sacrifices a nontoken creature of their choice.",
            targets: &[],
            effect: sacrifice_nontoken,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Each opponent sacrifices a creature token of their choice.",
            targets: &[],
            effect: sacrifice_token,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Each opponent sacrifices a planeswalker of their choice.",
            targets: &[],
            effect: sacrifice_planeswalker,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SHEOLDREDS_EDICT };
