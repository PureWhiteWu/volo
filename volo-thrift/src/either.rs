use std::future::Future;

use motore::{layer::Layer, Service, ServiceExt};

#[derive(Clone, Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<S, A, B> Layer<S> for Either<A, B>
where
    A: Layer<S>,
    B: Layer<S>,
{
    type Service = Either<A::Service, B::Service>;

    fn layer(self, inner: S) -> Self::Service {
        match self {
            Either::A(layer) => Either::A(layer.layer(inner)),
            Either::B(layer) => Either::B(layer.layer(inner)),
        }
    }
}

impl<A, B, Cx, Req> Service<Cx, Req> for Either<A, B>
where
    Req: 'static + Send,
    Cx: Send + 'static,
    A: Service<Cx, Req> + Send + 'static,
    B: Service<Cx, Req, Response = A::Response> + Send + 'static,
    A::Error: Into<crate::Error>,
    B::Error: Into<crate::Error>,
{
    type Response = A::Response;

    type Error = crate::Error;

    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'cx
    where
    Cx: 'cx,
    Self: 'cx;

    fn call<'cx, 's>(&'s mut self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        async move {
            match self {
                Either::A(s) => s.call(cx, req).await.map_err(|e| e.into()),
                Either::B(s) => s.call(cx, req).await.map_err(|e| e.into()),
            }
        }
    }
}
