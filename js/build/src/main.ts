import { Command } from 'commander';
import { readFile, writeFile } from "node:fs/promises";
import { simpleGit } from 'simple-git';
import { EOL } from "node:os";
import { Octokit } from 'octokit';
import { sync as spawnSync } from "cross-spawn";
import path from "node:path";
import { mkdirSync, readFileSync } from "fs";
import { copyFileSync, writeFileSync } from "node:fs";
import * as core from '@actions/core';
import { SpawnSyncOptions } from "child_process";

const program = new Command();

program
    .name('gauntlet-build')
    .description('Gauntlet Build Tool')

program.command('publish-init')
    .action(async () => {
        await doPublishInit()
    });

program.command('publish-linux')
    .action(async () => {
        await doPublishLinux()
    });

program.command('publish-macos')
    .action(async () => {
        await doPublishMacOS()
    });

program.command('publish-windows')
    .action(async () => {
        await doPublishWindows()
    });

program.command('publish-final')
    .action(async () => {
        await doPublishFinal()
    });

program.command('build-linux')
    .action(async () => {
        await doBuildLinux()
    });

program.command('build-macos')
    .action(async () => {
        await doBuildMacOS()
    });

program.command('build-windows')
    .action(async () => {
        await doBuildWindows()
    });

await program.parseAsync(process.argv);

async function doBuild(projectRoot: string, arch: string) {
    console.log("Building Gauntlet...")

    build(projectRoot, arch)
}

async function doPublishInit() {
    console.log("Publishing Gauntlet... Initiating...")

    const projectRoot = getProjectRoot()

    const githubReleaseId = process.env.PROVIDED_GITHUB_RELEASE_ID;

    if (githubReleaseId) {
        core.setOutput("github-release-id", `${githubReleaseId}`)
    } else {
        const { newVersion, releaseNotes } = await makeRepoChanges(projectRoot);

        const releaseId = await createRelease(newVersion, releaseNotes);

        console.log(`GitHub release id: ${releaseId}`)

        core.setOutput("github-release-id", `${releaseId}`)
    }
}

async function doPublishLinux() {
    const projectRoot = getProjectRoot()

    const git = simpleGit(projectRoot);

    console.log("git pull...")
    await git.pull()

    const arch = 'x86_64-unknown-linux-gnu';

    buildSize(projectRoot, arch)

    const { fileName, filePath } = packageForLinux(projectRoot, arch)

    await addFileToRelease(filePath, fileName)
}

async function doBuildLinux() {
    const arch = 'x86_64-unknown-linux-gnu';
    const projectRoot = getProjectRoot();

    await doBuild(projectRoot, arch)
    packageForLinux(projectRoot, arch)
}

async function doPublishMacOS() {
    const projectRoot = getProjectRoot();

    const git = simpleGit(projectRoot);

    console.log("git pull...")
    await git.pull()

    const arch = 'aarch64-apple-darwin';

    buildSize(projectRoot, arch)

    const { fileName, filePath } = await packageForMacos(projectRoot, arch, true, true)

    await addFileToRelease(filePath, fileName)
}

async function doBuildMacOS() {
    const projectRoot = getProjectRoot();
    const arch = 'aarch64-apple-darwin';

    await doBuild(projectRoot, arch)
    await packageForMacos(projectRoot, arch, false, false)
}

async function doPublishWindows() {
    const projectRoot = getProjectRoot();

    const git = simpleGit(projectRoot);

    console.log("git pull...")
    await git.pull()

    const arch = 'x86_64-pc-windows-msvc';

    buildSize(projectRoot, arch)

    const { fileName, filePath } = await packageForWindows(projectRoot, arch)

    await addFileToRelease(filePath, fileName)
}

async function doBuildWindows() {
    const projectRoot = getProjectRoot();
    const arch = 'x86_64-pc-windows-msvc';

    await doBuild(projectRoot, arch)
    await packageForWindows(projectRoot, arch)
}

async function doPublishFinal() {
    const projectRoot = getProjectRoot()

    const git = simpleGit(projectRoot);

    console.log("git pull...")
    await git.pull()

    console.log("Publishing Gauntlet npm packages...")

    buildJs(projectRoot)

    await publishNpmPackage(projectRoot)
}

function build(projectRoot: string, arch: string) {
    buildJs(projectRoot)

    buildRust(projectRoot, arch)
}

function buildSize(projectRoot: string, arch: string) {
    buildJs(projectRoot)

    buildRustSize(projectRoot, arch)
}

function buildRust(projectRoot: string, arch: string) {
    console.log("Building rust...")
    spawnWithErrors('cargo', ['build', '--release', '--features', 'release', '--target', arch], {
        cwd: projectRoot
    });
}

function buildRustSize(projectRoot: string, arch: string) {
    console.log("Building rust...")
    spawnWithErrors('cargo', ['build', '--profile', 'release-size', '--features', 'release', '--target', arch], {
        cwd: projectRoot
    });
}

function buildJs(projectRoot: string) {
    console.log("Building js...")
    spawnWithErrors('npm', ['run', 'build'], { cwd: projectRoot });
}

function getProjectRoot(): string {
    const projectRoot = path.resolve(process.cwd(), '..', '..');
    console.log("Project root: " + projectRoot)
    return projectRoot
}

async function makeRepoChanges(projectRoot: string): Promise<{ releaseNotes: string; newVersion: number; }> {
    const git = simpleGit(projectRoot);

    console.log("Reading version file...")
    const oldVersion = await readVersion(projectRoot)

    const newVersion = oldVersion + 1;

    console.log("Writing version file...")
    await writeVersion(projectRoot, newVersion)

    console.log("Reading changelog file...")
    const changelogFilePath = path.join(projectRoot, "CHANGELOG.md");
    const changelogRaw = await readFile(changelogFilePath, { encoding: "utf-8" });

    const notesForCurrentRelease: string[] = []
    const newChangelog: string[] = []

    let section: "before" | "unreleased" | "after" = "before";
    for (const line of changelogRaw.split(EOL)) {
        switch (section) {
            case "before": {
                if (line.match(/^## \[Unreleased]/) != null) {
                    section = "unreleased"

                    const date = new Date();
                    const year = date.getFullYear();
                    const month = `0${date.getMonth() + 1}`.slice(-2);
                    const day = `0${date.getDate()}`.slice(-2);

                    const formattedDate = `${year}-${month}-${day}`;

                    newChangelog.push(line)
                    newChangelog.push("")
                    newChangelog.push(`## [${newVersion}] - ${formattedDate}`)
                } else {
                    newChangelog.push(line)
                }
                break;
            }
            case "unreleased": {
                newChangelog.push(line)
                if (line.match(/^## /) != null) {
                    section = "after"
                } else {
                    notesForCurrentRelease.push(line)
                }
                break;
            }
            case "after": {
                newChangelog.push(line)
                break;
            }
        }
    }

    console.log("Writing changelog file...")
    await writeFile(changelogFilePath, newChangelog.join(EOL))

    console.log("git add all files...")
    await git.raw('add', '-A')
    console.log("git commit...")
    await git.commit(`Prepare for v${newVersion} release`);
    console.log("git add version tag...")
    await git.addTag(`v${newVersion}`)
    console.log("git push...")
    await git.push()
    console.log("git push tags...")
    await git.pushTags();

    return {
        newVersion,
        releaseNotes: notesForCurrentRelease.join('\n'),
    }
}

function packageForLinux(projectRoot: string, arch: string): { filePath: string; fileName: string } {
    const releaseDirPath = path.join(projectRoot, 'target', arch, 'release');
    const assetsDirPath = path.join(projectRoot, 'assets', 'linux');

    const sourceExecutableFilePath = path.join(releaseDirPath, 'gauntlet');
    const sourceDesktopFilePath = path.join(assetsDirPath, 'gauntlet.desktop');
    const sourceServiceFilePath = path.join(assetsDirPath, 'gauntlet.service');
    const sourceLogoFilePath = path.join(assetsDirPath, 'icon_256.png');

    const bundleDir = path.join(releaseDirPath, 'archive');

    const targetExecutableFileName = 'gauntlet';
    const targetExecutableFilePath = path.join(bundleDir, targetExecutableFileName);

    const targetDesktopFileName = 'gauntlet.desktop';
    const targetDesktopFilePath = path.join(bundleDir, targetDesktopFileName);

    const targetServiceFileName = 'gauntlet.service';
    const targetServiceFilePath = path.join(bundleDir, targetServiceFileName);

    const targetLogoFileName = 'gauntlet.png';
    const targetLogoFilePath = path.join(bundleDir, targetLogoFileName);

    const archiveFileName = 'gauntlet-x86_64-linux.tar.gz';
    const archiveFilePath = path.join(bundleDir, archiveFileName);

    mkdirSync(bundleDir)

    copyFileSync(sourceExecutableFilePath, targetExecutableFilePath)
    copyFileSync(sourceDesktopFilePath, targetDesktopFilePath)
    copyFileSync(sourceServiceFilePath, targetServiceFilePath)
    copyFileSync(sourceLogoFilePath, targetLogoFilePath)

    spawnWithErrors(`tar`, ['-czvf', archiveFileName, targetExecutableFileName, targetDesktopFileName, targetServiceFileName, targetLogoFileName], {
        cwd: bundleDir
    })

    return {
        filePath: archiveFilePath,
        fileName: archiveFileName
    }
}

async function packageForMacos(projectRoot: string, arch: string, sign: boolean, notarize: boolean): Promise<{ filePath: string; fileName: string }> {
    const releaseDirPath = path.join(projectRoot, 'target', arch, 'release');
    const sourceExecutableFilePath = path.join(releaseDirPath, 'gauntlet');
    const outFileName = "gauntlet-aarch64-macos.dmg"
    const outFilePath = path.join(releaseDirPath, outFileName);

    const assetsDirPath = path.join(projectRoot, 'assets', 'macos');
    const sourceInfoFilePath = path.join(assetsDirPath, 'Info.plist');
    const sourceIconFilePath = path.join(assetsDirPath, 'AppIcon.icns');
    const dmgBackground = path.join(assetsDirPath, 'dmg-background.png');
    const entitlementsPath = path.join(assetsDirPath, 'entitlements.plist');

    const bundleDir = path.join(releaseDirPath, 'Gauntlet.app');
    const contentsDir = path.join(bundleDir, 'Contents');
    const macosContentsDir = path.join(contentsDir, 'MacOS');
    const resourcesContentsDir = path.join(contentsDir, 'Resources');
    const targetExecutableFilePath = path.join(macosContentsDir, 'Gauntlet');
    const targetInfoFilePath = path.join(contentsDir, 'Info.plist');
    const targetIconFilePath = path.join(resourcesContentsDir, 'AppIcon.icns');

    const version = await readVersion(projectRoot)

    mkdirSync(bundleDir)
    mkdirSync(contentsDir)
    mkdirSync(macosContentsDir)
    mkdirSync(resourcesContentsDir)

    copyFileSync(sourceExecutableFilePath, targetExecutableFilePath)
    copyFileSync(sourceInfoFilePath, targetInfoFilePath)
    copyFileSync(sourceIconFilePath, targetIconFilePath)

    const infoSource = readFileSync(targetInfoFilePath, 'utf8');
    const infoResult = infoSource.replace('__VERSION__', `${version}.0.0`);
    writeFileSync(targetInfoFilePath, infoResult,'utf8');

    const signKeyPath = path.join(releaseDirPath, 'signKey.pem');
    const signCertPath = path.join(releaseDirPath, 'signCert.pem');
    const connectApiKeyPath = path.join(releaseDirPath, 'connectApiKey.json');

    const signKeyContent = process.env.APPLE_SIGNING_KEY_PEM;
    const signCertContent = process.env.APPLE_SIGNING_CERT_PEM;
    const connectApiKeyContent = process.env.APP_STORE_CONNECT_KEY;

    if (sign) {
        const key = JSON.parse(signKeyContent!!).content;
        const cert = JSON.parse(signCertContent!!).content;

        writeFileSync(signKeyPath, key);
        writeFileSync(signCertPath, cert);

        spawnWithErrors(`rcodesign`, [
            'sign',
            '--pem-file',
            signKeyPath,
            '--pem-file',
            signCertPath,
            '--for-notarization',
            '--entitlements-xml-file',
            entitlementsPath,
            bundleDir
        ], {
            cwd: releaseDirPath
        })
    }

    spawnWithErrors(`create-dmg`, [
        '--volname', 'Gauntlet Installer',
        '--window-size', '660', '400',
        '--background', dmgBackground,
        '--icon', "Gauntlet.app", '180', '170',
        '--icon-size', '160',
        '--app-drop-link', '480', '170',
        '--hide-extension', 'Gauntlet.app',
        outFileName,
        bundleDir
    ], {
        cwd: releaseDirPath
    })

    if (sign) {
        spawnWithErrors(`rcodesign`, [
            'sign',
            '--pem-file',
            signKeyPath,
            '--pem-file',
            signCertPath,
            '--for-notarization',
            outFilePath
        ], {
            cwd: releaseDirPath
        })
    }

    if (notarize) {
        writeFileSync(connectApiKeyPath, connectApiKeyContent!!);

        spawnWithErrors(`rcodesign`, [
            'notary-submit',
            '--api-key-file',
            connectApiKeyPath,
            '--staple',
            outFilePath
        ], {
            cwd: releaseDirPath
        })
    }

    return {
        filePath: outFilePath,
        fileName: outFileName
    }
}

async function packageForWindows(projectRoot: string, arch: string): Promise<{ filePath: string; fileName: string }> {
    const releaseDirPath = path.join(projectRoot, 'target', arch, 'release');
    const sourceExecutableFilePath = path.join(releaseDirPath, 'gauntlet.exe');
    const outFileName = "gauntlet-x86_64-windows.msi"
    const outFilePath = path.join(releaseDirPath, outFileName);

    const assetsDirPath = path.join(projectRoot, 'assets', 'windows');
    const sourceWxsFilePath = path.join(assetsDirPath, 'main.wxs');
    const iconFilePath = path.join(projectRoot, 'assets', 'linux', 'icon_256.png');

    const targetWxsFilePath = path.join(releaseDirPath, 'main.wxs');
    const targetIconFilePath = path.join(releaseDirPath, 'icon.ico');

    const version = await readVersion(projectRoot)

    copyFileSync(sourceWxsFilePath, targetWxsFilePath)

    spawnWithErrors("magick.exe", [iconFilePath, '-define', 'icon:auto-resize=256,128,48,32,16', targetIconFilePath], {
        cwd: releaseDirPath
    })

    spawnWithErrors("wix", [
        'build',
        targetWxsFilePath,
        '-out', outFilePath,
        '-define', `TargetBinaryPath=${sourceExecutableFilePath}`,
        '-define', `TargetIconPath=${targetIconFilePath}`,
        '-define', `TargetVersion=${version}.0`,
        '-ext', "WixToolset.Util.wixext",
        '-arch', "x64",
    ], {
        cwd: releaseDirPath
    })

    return {
        filePath: outFilePath,
        fileName: outFileName
    }
}


async function publishNpmPackage(projectRoot: string) {
    const version = await readVersion(projectRoot)

    const apiProjectPath = path.join(projectRoot, "js", "api");

    console.log("Bump version for api subproject...")
    spawnWithErrors('npm', ['version', `0.${version}.0`], { cwd: apiProjectPath })

    console.log("Publishing npm api package...")
    spawnWithErrors('npm', ['publish'], { cwd: apiProjectPath })
}

async function createRelease(newVersion: number, releaseNotes: string): Promise<number> {
    const octokit = getOctokit();

    console.log("Creating github release...")

    const response = await octokit.rest.repos.createRelease({
        ...getGithubRepo(),
        tag_name: `v${newVersion}`,
        target_commitish: 'main',
        name: `v${newVersion}`,
        body: releaseNotes,
        draft: true // release needs to be undrafted manually after each release
    });

    return response.data.id;
}

async function addFileToRelease(filePath: string, fileName: string) {
    const octokit = getOctokit();

    const response = await octokit.rest.repos.getRelease({
        ...getGithubRepo(),
        release_id: getGithubReleaseId(),
    });

    const fileBuffer = await readFile(filePath);

    console.log("Uploading executable as github release asset...")
    await octokit.rest.repos.uploadReleaseAsset({
        ...getGithubRepo(),
        release_id: response.data.id,
        origin: response.data.upload_url,
        name: fileName,
        headers: {
            "Content-Type": "application/octet-stream",
        },
        data: fileBuffer as any,
    })
}

function getOctokit() {
    return new Octokit({
        auth: process.env.GITHUB_TOKEN,
    })
}
function getGithubRepo() {
    return {
        owner: 'project-gauntlet',
        repo: 'gauntlet',
    }
}

function getGithubReleaseId() {
    return Number(process.env.GITHUB_RELEASE_ID!!)
}

async function readVersion(projectRoot: string): Promise<number> {
    const versionFilePath = path.join(projectRoot, "VERSION");
    const versionRaw = await readFile(versionFilePath, { encoding: "utf-8" });
    return Number(versionRaw.trim())
}

async function writeVersion(projectRoot: string, version: number) {
    const versionFilePath = path.join(projectRoot, "VERSION");
    await writeFile(versionFilePath, `${version}`)
}

function spawnWithErrors(command: string, args: string[], options: SpawnSyncOptions) {
    console.log(`running ${command} ${args}`)

    const npmRunResult = spawnSync(command, args, { ...options, stdio: "inherit", });

    if (npmRunResult.status !== 0) {
        throw new Error(`Unable to run ${command} ${args}, status: ${JSON.stringify(npmRunResult, null, 2)}`);
    }
}