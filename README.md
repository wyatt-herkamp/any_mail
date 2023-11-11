# Any_Mail

A simple way to send emails from your application

## In Progress

This Library is still in progress and most features are not implemented

## Requires Rust Nightly

Currently this project is using Rust Async Traits internally
so Rust Nightly is required.

## Supported Mail Services

- [x] SMTP - Direct SMTP Connection VIA [Lettre](https://github.com/lettre/lettre)
- [ ] [MailWhale](https://mailwhale.dev/)
- [ ] [MailGun](https://www.mailgun.com/)
- Missing your Mail Service? Make a PR!

## Features

- Setting Types Built for Configs
- (Coming Soon) Pull Data from Environment Variables
- Built in Templating
- Do not worry about errors or a lack of email server

## Design

This works by starting a "service" or a green thread.
That holds a channel reciever of emails that will send them.
Then you will have an access that pushes the items to the queue.

No need to worry about blocking while the email sends or errors.
