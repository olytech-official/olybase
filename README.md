<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Apache License][license-shield]][license-url]


<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/olytech-official/olybase">
    <img src="images/logo.svg" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Olybase</h3>

  <p align="center">
    A modular, high-performance Axum web framework wrapper with optional features for JWT, TLS, WebSockets, Storage, and more!
    <br />
    &middot;
    <a href="https://github.com/olytech-official/olybase/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/olytech-official/olybase/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With & Workspace Crates</a></li>
        <li><a href="#features">Features & Cargo Flags</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project
Olybase is a modular, extensible meta-framework and wrapper built on top of **Axum** and **Tokio**. It aims to eliminate boilerplate when setting up production-ready Rust web services by providing cleanly decoupled, opt-in crates for security, web routing, real-time websockets, and storage layers.

Whether you need a lightweight microservice or a full-featured backend complete with TLS auto-generation, JWT authentication, refresh tokens, rate-limiting, and compression, Olybase lets you opt into only what you use via Cargo features.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


### Built With & Workspace Crates

This section should list any major frameworks/libraries used to bootstrap your project. Leave any add-ons/plugins for the acknowledgements section. Here are a few examples.

Olybase is structured as a Cargo workspace with dedicated modular crates:
* `olybase-core` - SoonTM
* `olybase-security` - SoonTM
* `olybase-web` - SoonTM
* `olybase-ws` - SoonTM
* `olybase-storage` - SoonTM

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Features & Cargo Flags

Control your binary size and dependencies by enabling only the features you need in your `Cargo.toml`:

| Feature Flag | Enables Sub-crates / Functionality |
| :--- | :--- |
| `web` | Enables `olybase-web` routing and server capabilities |
| `tls` | Enables web layer TLS support (`olybase-web/tls`) |
| `tls-autogen` | Enables automatic TLS certificate generation |
| `compression` | Enables response compression middleware |
| `rate-limit` | Enables request rate-limiting utilities |
| `tracing` | Enables structured logging and tracing support |
| `security` | Enables `olybase-security` framework |
| `jwt-autogen` | Enables automatic JWT secret / token generation tools |
| `refresh-token` | Enables secure refresh token workflows |
| `password-hasher` | Enables secure password hashing utilities |
| `ws` | Enables `olybase-ws` for WebSockets |
| `storage` | Enables `olybase-storage` for data persistence |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

To get a local copy up and running, follow these steps.

### Prerequisites

This is an example of how to list things you need to use the software and how to install them.
* [Install Rust](https://www.rust-lang.org/tools/install)

### Installation

_Below is an example of how you can instruct your audience on installing and setting up your app. This template doesn't rely on any external dependencies or services._

1. Clone the repo
   ```sh
   git clone https://github.com/olytech-official/olybase.git
   ```
2. Navigate into the directory
   ```sh
   cd olybase
   ```
3. Add Olybase to your project dependency list (with your chosen features) in your app's Cargo.toml:
  ```sh
   [dependencies]
   olybase = { version = "0.1.0", features = ["web", "security", "jwt-autogen", "tls"] }
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

Here is a quick example of how you can set up a basic server using Olybase:

```rust
/// todo
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [x] Establish modular workspace architecture (core, security, web, ws, storage)
- [x] Implement optional security features (JWT, Password Hashing, Refresh Tokens)
- [x] Implement web middleware features (Tracing, Rate-limiting, Compression)
- [ ] Add a Database storage option (postgres and sqlite via sqlx)
- [ ] Add comprehensive integration test suite across all workspace crates
- [ ] Publish official crates to crates.io

See the [open issues](https://github.com/olytech-official/olybase/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request


<!-- LICENSE -->
## License

Distributed under the Apache License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Project Link: [https://github.com/olytech-official/olybase](https://github.com/olytech-official/olybase)

<p align="right">(<a href="#readme-top">back to top</a>)</p>
