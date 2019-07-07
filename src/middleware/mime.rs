use crate::error::PointercrateError;
use actix_web::{
    middleware::{Middleware, Response, Started},
    HttpRequest, HttpResponse, Result,
};
use log::{trace, warn};
use mime::Mime;

#[derive(Debug, Copy, Clone)]
pub struct MimeProcess;

#[derive(Debug)]
pub struct Accept(pub Vec<Mime>);
#[derive(Debug)]
pub struct ContentType(pub Option<Mime>);

impl<S> Middleware<S> for MimeProcess {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        trace!("Received request {:?}", req);

        let content_type = match header!(req, "Content-Type") {
            Some(cntt) =>
                Some(cntt.parse::<Mime>().map_err(|_| {
                    PointercrateError::InvalidHeaderValue {
                        header: "Content-Type",
                    }
                })?),
            None => None,
        };

        let accept = match header!(req, "Accept") {
            Some(accepts) =>
                accepts
                    .split(',')
                    .map(|accept| {
                        accept.trim().parse::<Mime>().map_err(|err| {
                            warn!("Invalid accept header mime type: {} - {}", accept, err);

                            PointercrateError::InvalidHeaderValue { header: "Accept" }
                        })
                    })
                    .collect::<std::result::Result<Vec<Mime>, PointercrateError>>()?,
            None => Vec::new(),
        };

        req.extensions_mut().insert(Accept(accept));
        req.extensions_mut().insert(ContentType(content_type));

        Ok(Started::Done)
    }

    fn response(&self, _: &HttpRequest<S>, resp: HttpResponse) -> Result<Response> {
        trace!("Response is {:?}", resp);

        Ok(Response::Done(resp))
    }
}
