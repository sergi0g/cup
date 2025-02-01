# Cup Documentation

## Architecture

The docs are built with [Nextra](https://nextra.site). We use [Bun](https://bun.sh) as a package manager and Node.js as a runtime (Next.js and Bun don't play well together at the moment). Docs pages are written in [MDX](https://mdxjs.com) and any custom components are written in TypeScript with TSX.

## Development

Prerequisites:

- A recent Node.js version (22 recommended)
- [Bun](https://bun.sh)

```bash
git clone https://github.com/sergi0g/cup
cd cup/docs
bun install
```

You're ready to go!

## Scripts

The available scripts are:

- `bun dev` starts the development server. Note that making changes to MDX pages will probably require a full reload.
- `bun run build` creates a static production build, ready to be deployed.
- `bun lint` checks for errors in your code.
- `bun fmt` formats your code with Prettier, so it becomes... prettier.

## Contributing

Our documentation is always evolving, so, we constantly need to update this repository with new guides and configuration options. If you have any ideas of a guide or suggestions on how to improve them, feel free to open a pull request or create an issue. All contributions are welcome!

## License

The documentation is licensed under the MIT License. TL;DR — You are free to use, copy, modify, merge, publish, distribute, sublicense, and sell copies of the software. However, the software is provided "as is," without warranty of any kind. You must include the original license in all copies or substantial portions of the software.
