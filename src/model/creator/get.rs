use super::{Creator, Creators};
use crate::{
    citext::CiStr, context::RequestContext, error::PointercrateError, model::Demon, operation::Get,
    schema::creators, Result,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Creators {
    fn get(name: &'a CiStr, ctx: RequestContext) -> Result<Self> {
        super::creators_of(name)
            .load(ctx.connection())
            .map(Creators)
            .map_err(PointercrateError::database)
    }
}

impl Get<(i16, i32)> for Creator {
    fn get((demon_position, player_id): (i16, i32), ctx: RequestContext) -> Result<Self> {
        let demon = Demon::get(demon_position, ctx)?;

        creators::table
            .select((creators::demon, creators::creator))
            .filter(creators::demon.eq(&demon.name))
            .filter(creators::creator.eq(&player_id))
            .get_result(ctx.connection())
            .map_err(PointercrateError::database)
    }
}
