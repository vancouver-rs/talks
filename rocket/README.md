Notes from a talk given on 2021-01-20. Some of the links don't quite line up with the right lines of code due to mismatched commit SHAs, if it doesn't make sense switch it back to mainline branch.

Tour of a Rocket app:

App is at https://github.com/idevgames/uDevGames.com

- Uses Rocket (master branch).
- The site is unfinished (project pivot, sadly this isn't deployed to the wild anywhere, but if you're curious how it might be done I have some deployment notes for running Rust services on Linux machines at https://github.com/mysteriouspants/mysteriousbot#deploying).
- Features
  - HTML templating using Tera https://github.com/idevgames/uDevGames.com/tree/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/templates
  - database persistence using SQLite and Diesel as an ORM
    - some of the types are rather wordy, so i've aliased them https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/db.rs
    - migrations are handled by diesel_migrate
      - https://github.com/idevgames/uDevGames.com/tree/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/migrations
      - https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/db.rs#L25-L30
    - a light layer of abstraction sits on top of diesel called a model https://github.com/idevgames/uDevGames.com/tree/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/models
  - OAuth with Github https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/gh_oauth.rs#L58-L198
  - Sessions persisting user state
    - logging someone in https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/gh_oauth.rs#L58-L78
    - pulling "a user might be logged in" out of a request and providing it to a request handler:
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/template_helpers/user_optional.rs
    - pulling "an admin must be logged in or we're going to have a problem" out of a request and providing it to a request handler:
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/template_helpers/admin_only.rs
      note that if you aren't an admin this will produce a NotAuthorized response like you would expect.
      just decorate these onto handler method arguments and they magically work
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/jams.rs#L33-L42
    - logging someone out https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/gh_oauth.rs#L200-L215
  - web forms!
    - write forms in html like you're used to
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/templates/edit_jam.html.tera
    - write a struct which describes what you're getting out of the form, decorate it with #[derive(FromForm)]
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/jams.rs#L76-L88
    - put that in your handler as the data extractor, it just works
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/jams.rs#L90-L133
    - fault finding: bad input misses the router and so it craters with a weird error. it's not fun to debug, it would be worse for a customer
    - fault finding: FormForm is kinda weird, serde::Deserialize would probably be better if it could be made to work
- Rocket stuff
  - starting a server is pretty easy, throw your global state at it, tell it about your handlers, then it just goes
    https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/serve.rs
  - imho rocket tries to be too clever, guessing configuration based on environment variables and such. I ended up overriding all this because I wanted more careful control over how my application was configured.
  - you can serve arbitray files from rocket really easily.
    https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/serve.rs#L51
  - Rocket mainline (and someday Rocket 0.5.0) is fully async, on Tokio
    - So you can do spiffy things like external web requests in handler code and it doesn't block your processing of other requests
      here i'm using reqwest like a scrub
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/gh_oauth.rs#L124-L146
    - You can return streaming content without blocking other requests.
      Here I return a file streaming from disk. If it's particularly large it could take a while to download, but it wouldn't block a processing thread because it's using Tokio's File IO under the hood.
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/attachments.rs
    - Actix-web is also fully async now (3.0.0), and Actix-rt uses Tokio under the hood. https://crates.io/crates/actix-web
    - I'm not a fan of Actix-rt's model though, I rather wish they'd expose the full Tokio runtime and let the Actix actor system be a bonus if you want it. If wishes were horses.
    - Actix-web is going Tokio 1.0 in version 4.0, currently in beta. So, soon?
    - The other async web framework to keep an eye on is Warp (disclaimer, seanmonstar works at the same company as me). https://crates.io/crates/warp
      fun fact, warp just went 0.3 and thus takes Tokio 1.0 yesterday, 19 January 2021!
    - Another web framework to keep an eye on is Conduit - the one that runs crates.io.
      It's not documented, so not intended for public consumption. Yet?
      https://crates.io/crates/conduit
- What did Rocket absolutely not do well?
  - CSS - I prefer to use SASS because when I last did webdev full time I was big into SASS.
  - I ended up using Brunch to compile my JS and CSS assets
    https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/package.json
    https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/brunch-config.js
    this lets me vendor my Twitter Bootstrap and jQuery files from NPM
    there better ways to do things in 2021, but jQuery, jQuery-ujs, and Twitter Bootstrap are "good enough" for my purposes. You could go all the way and embed a complicated client-side application that gets served by Rocket - Rocket speaks JSON quite happily and can be easily adapted for that. I would suggest that Rocket and really all the Rust web frameworks are better at JSON than they are HTML right now.
  - Errors - but I'm pretty sure error handling is the best worst part of Rust anyway, so this isn't really a super big deal
    - I wrote a rather large unified error type for my handlers to return
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/controllers/mod.rs#L13-L71
      It has a *lot* of boilerplate for translating itself to Rocket error statuses.
    - These statuses are sent to "error catchers."
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/serve.rs#L52-L57
      https://github.com/idevgames/uDevGames.com/blob/6d33bd3f5c636aca1718f14eed3bb11a7e8f44e6/src/error_handlers.rs
    - Note that by the time we make it to the error catcher we've lost the original error entirely. There is no way to say something like "the date format you used is incorrect please try again" unless you do it _entirely_ manually and out of band of the normal error catching system.
  - Return types
    - Template, Redirect, Result, Responder, what?
    - Everything a handler returns _must implement_ Responder.
    - Rocket has helpfully implemented Responder for some basics, such as Result<impl Responder, impl Responder>
    - This gets you _most_ of the way to done
    - Except when your handler needs to return one of three different things!
      - Template in the happy path
      - Redirect in the not authorized path
      - Error in the "oh my goodness the database has gone on holiday" path
      - this turns into some abomination such as Result<Result<Template, Redirect>, HandlerError>
      - Ok(Err(Redirect::to("/foo"))), but it isn't an error, really... gross
- Speed
  - These things are all so fast, unless you have some specific performance requirements at scale Rocket or Actix or anything is going to be far and away faster than you need.
- Other resources I trawl looking for new toys
  - Awesome Rust     https://awesome-rust.com/
  - Are We Web Yet?  https://www.arewewebyet.org/ *whoever made this site is really optimisitic
  
Takeaway opinion:
- Rust is relatively immature for the web compared to something like Rails or Django, but it's really strong at building programmatic APIs. It speaks JSON really well.
- Trying to build a human-facing website served directly by a Rust server is a very different story. There are little pain points and the lack of duck-typing makes some of the more complicated workflows endemic to good UX really overwrought with lots of Rust code to support them.
- I believe that Rust is at a level of maturity for building websites that you ought to - it's 75% of the way there, and by having more people working on it and reporting bugs and making improvements it'll get the rest of the way there to maturity.

Anecdote, the one friend I roped into contributing some to this project remarked that he was utterly uninterested in the web side but thought that Rust was neat. So that's cool.
