# Contribute to the Lumper project

Thank you for investing your time in contributing to our project!

In this guide you will get an overview of the contribution workflow from opening an issue, creating a PR, committing your changes, and merging the PR.

## Getting started

### Create a new issue

First, you have to create an issue in the `virt-do/lumper` repository describing what you are going to do. Please be as precise as possible to allow other contributors to understand what you will implement. It will make it easier for other developers if you need help or if your issue does not make sense.

> See [Github - Creating an issue](https://docs.github.com/en/issues/tracking-your-work-with-issues/creating-an-issue).

### Fork the repository

Then, you have to fork the `virt-do/lumper` in order to do your changes freely without affecting the original project.

> See [Github - Fork a repo](https://docs.github.com/en/issues/tracking-your-work-with-issues/creating-an-issue).

You are now able to work in your own `lumper`, you juste have to clone your fork.

## Commits convention

### Conventional commits

When committing your changes, please use a commit message with the following format:
```
<subsystem>: <description>

[optional body]

[optional footer(s)]
```

For example: `vmm: Use builder pattern to configure the VMM`

Make sure to be as clear and concise as possible. This will make easier for people to contribute to this project, by allowing them to explore a more structured commit history.

> See [Kernel - Describe your changes](https://docs.kernel.org/process/submitting-patches.html#describe-your-changes).

### SOB

Please make sur to include a sign-off message in your commit.
It allows to prove that your are the commit's author.

You just have to had the following in your commit footer :
```
Signed-off-by: John Doe <john.doe@example.com>
```
Or simply use the `-s`, `-signoff` option : `git commit -s -m "This is my commit message"`, this will use your default git configuration which is found in `.git/config`.

## Pull request quality

Your pull request must respect the following requirements :

- Code quality : comments, idiomatic, formatting, etc
- Logical split : [Orthogonal commits](https://docs.kernel.org/process/submitting-patches.html#separate-your-changes) and always buildable
- Documentation and unit tests

> NOTE : Mark your PR as WIP if it’s not ready to be reviewed

## Submit your Pull Request

> NOTE : Please make sure to rebase your fork branch with the virt-do:main branch before submitting

When your changes are finished and you have follow the rules stated above, you juste have to submit you Pull Request. 

> See [Github - Creating a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request).

## Your PR is merged!

Congratulations! 🎉👏

The `lumper` team thanks you. 😉