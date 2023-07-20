use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{
        middleware::{
            ConstrainedMiddleware, Middleware, MiddlewareBuilder, MiddlewareLayerBuilder,
            ProcedureKind, ResolverLayer,
        },
        procedure::BuildProceduresCtx,
        FutureMarkerType, HasResolver, ProcedureDataType, RequestLayer, ResolverFunction,
        StreamMarkerType,
    },
    ExecError,
};

/// TODO: Explain
pub struct MissingResolver;

impl Default for MissingResolver {
    fn default() -> Self {
        Self
    }
}

mod private {
    use std::marker::PhantomData;

    pub struct Procedure<T, Err, TMiddleware> {
        pub(crate) resolver: T,
        pub(crate) mw: TMiddleware,
        pub(crate) phantom: PhantomData<Err>,
    }
}

pub(crate) use private::Procedure;

impl<TMiddleware, Err, T> Procedure<T, Err, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn new(resolver: T, mw: TMiddleware) -> Self {
        Self {
            resolver,
            mw,
            phantom: PhantomData,
        }
    }
}

macro_rules! resolver {
    ($func:ident, $kind:ident, $result_marker:ident) => {
        pub fn $func<R, RMarker>(self, resolver: R) -> Procedure<RMarker, Err, TMiddleware>
        where
            R: ResolverFunction<TMiddleware::LayerCtx, RMarker>,
            R::Result: RequestLayer<R::RequestMarker, Type = $result_marker>,
        {
            Procedure::new(resolver.into_marker(ProcedureKind::$kind), self.mw)
        }
    };
}

// Can only set the resolver or add middleware until a resolver has been set.
// Eg. `.query().subscription()` makes no sense.
impl<Err, TMiddleware> Procedure<MissingResolver, Err, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    resolver!(query, Query, FutureMarkerType);
    resolver!(mutation, Mutation, FutureMarkerType);
    resolver!(subscription, Subscription, StreamMarkerType);

    pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver, Err, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: Middleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver, Err, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }
}

impl<F, TArg, TResult, TResultMarker, Err, TMiddleware>
    Procedure<HasResolver<F, TMiddleware::LayerCtx, TArg, TResult, TResultMarker>, Err, TMiddleware>
where
    F: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: Type + DeserializeOwned + 'static,
    TResult: RequestLayer<TResultMarker> + 'static,
    TResultMarker: 'static,
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn build(
        self,
        key: Cow<'static, str>,
        ctx: &mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
    ) {
        let HasResolver(resolver, kind, _) = self.resolver;

        let m = match kind {
            ProcedureKind::Query => &mut ctx.queries,
            ProcedureKind::Mutation => &mut ctx.mutations,
            ProcedureKind::Subscription => &mut ctx.subscriptions,
        };

        let key_str = key.to_string();
        let type_def = ProcedureDataType::from_tys::<TMiddleware::Arg<TArg>, TResult::Result>(
            key,
            ctx.ty_store,
        )
        .expect("error exporting types"); // TODO: Error handling using `#[track_caller]`

        m.append(
            key_str,
            self.mw.build(ResolverLayer::new(move |ctx, input, _| {
                Ok((resolver)(
                    ctx,
                    serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                )
                .exec())
            })),
            type_def,
        );
    }
}
