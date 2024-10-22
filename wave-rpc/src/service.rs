pub trait Service {
    type Request;
    type Response;

    const ID: u32;
}
