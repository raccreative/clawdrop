
<p align="center">
  <img src="https://raccreative.s3.eu-central-1.amazonaws.com/media/logoHorizontal2.svg" alt="Raccreative Games" width="600"/>
</p>

  <a href="https://discord.com/invite/XTajeBv89n"><img src="https://img.shields.io/discord/733027681184251937.svg?style=flat&label=Join%20Discord&color=7289DA" alt="Join Discord"/>
  [![web](https://img.shields.io/badge/web-Raccreative%20Games-blue.svg)](https://raccreativegames.com)
  ![GitHub License](https://img.shields.io/github/license/raccreative/clawdrop)
  ![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
  [![Rust](https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white)](#)


---

# Clawdrop 🦀🎮🦝

**Clawdrop** is the official command-line tool for managing and publishing games on [Raccreative Games](https://raccreativegames.com).  
It allows developers to authenticate, upload builds and publish posts.

## Features

- Authenticate with Raccreative Games
- Upload new builds of your games
- Publish posts directly from your terminal
- Manage target game configuration
- Lightweight and fast, built in Rust 🦀

---

## Basic Usage  
```
clawdrop <command> [options]
```

## Documentation  

For all the details, check the [full documentation](https://raccreativegames.com/docs/clawdrop).  

---

## Clawdrop Commands

| Command   | Description                                                                 |
|-----------|-----------------------------------------------------------------------------|
| `doctor`  | Development environment diagnostics                                         |
| `auth`    | Authorization to use API Key from Raccreative (opens URL)                   |
| `logout`  | Remove API Key and log out                                                  |
| `list`    | Shows a list with the games you have permissions to upload builds           |
| `set`     | Sets a game via ID or URL slug to be the main target of clawdrop            |
| `unset`   | Removes the current game target                                             |
| `whereis` | Prints the current clawdrop executable location                             |
| `post`    | Publish a post for the target or specified game                             |
| `push`    | Upload a new build of an existing game to Raccreative Games                 |
| `help`    | Print this message or the help of the given subcommand(s)                   |

### Options

- `-h, --help` — Print help (see a summary with `-h`)  
- `-V, --version` — Print version

---

## Contributing  

Contributions are welcome!  
1. Fork the repository.  
2. Create a branch for your feature/fix (`git checkout -b feature/new-feature`).  
3. Commit your changes (`git commit -m 'Added new feature'`).  
4. Push to your branch (`git push origin feature/new-feature`).  
5. Open a Pull Request.  

## License  

This project is licensed under the [MIT License](LICENSE).  


