const {match} = require('assert');

module.exports = main;

const CHANGELOG_FILE = 'CHANGELOG.md';
const MAX_BODY_LENGTH = 125000;
const LOCAL = ['zh-CN', {timeZone: 'CST'}, ' (北京时间)'];


async function main({github, context, sha}) {
  // require modules
  const {execSync} = require('child_process');
  const fs = require('fs');

  // get the latest tag (first tag in the list)
  const {tag, tag_sha} = await get_latest_tag({github, context});

  // get the release body
  const {body, origin_body} = await get_release_body({execSync, fs, tag_sha, sha});

  // save release_body as artifact
  fs.writeFileSync('release_body.md', origin_body);

  // create a new release with the tag and commit message
  const name = `Release ${tag} by ${context.actor}`;
  const release = await github.rest.repos.createRelease({
    owner: context.repo.owner,
    repo: context.repo.repo,
    tag_name: tag,
    // target_commitish: sha,
    name: name,
    body: body,
    draft: false,
    prerelease: false,
  });
  const release_id = release.data.id;

  // get the artifacts
  const artifacts = await get_artifacts({github, context});

  // sleep for 1 second to make sure the release is created
  // await new Promise(r => setTimeout(r, 1000));

  // upload body_raw to the release
  await github.rest.repos.uploadReleaseAsset({
    owner: context.repo.owner,
    repo: context.repo.repo,
    release_id: release_id,
    name: 'release_body.md',
    data: origin_body,
  });

  // upload them to the release
  for (const {name, data} of artifacts) {
    await github.rest.repos.uploadReleaseAsset({
      owner: context.repo.owner,
      repo: context.repo.repo,
      release_id: release_id,
      name: name + '.zip',
      data: data,
    });
  }

  console.log(`Release ${tag} created successfully!`);
}

async function get_artifacts({github, context}) {
  // list all possible artifacts
  const artifacts = await github.rest.actions.listWorkflowRunArtifacts({
    owner: context.repo.owner,
    repo: context.repo.repo,
    run_id: context.runId,
  });
  let result = new Array();
  for (const {id, name} of artifacts.data.artifacts) {
    const artifact = await github.rest.actions.downloadArtifact({
      owner: context.repo.owner,
      repo: context.repo.repo,
      artifact_id: id,
      archive_format: 'zip'
    });
    result.push({name, data: artifact.data});
  }
  return result;
}

async function get_latest_tag({github, context}) {
  const response = await github.rest.repos.listTags({
    owner: context.repo.owner,  // owner of the repo
    repo: context.repo.repo,    // name of the repo
    per_page: 1                 // only need the first tag
  });
  const {name, commit: {sha}} = response.data[0];
  // extract the version number from the tag (v1.2.3.4 => major=1, minor=2,
  // patch=3, build=4) need to convert the version numbers from string to number
  const [major, minor, patch, build] = name.substr(1).split('.').map(Number);
  console.log(`ma: ${major}, mi: ${minor}, p: ${patch}, b: ${build};`);
  console.log(`runNumber: ${context.runNumber}`);
  // increment the patch number and change build to running number
  const tag = `v${major}.${minor}.${patch + 1}.${context.runNumber}`;
  return {tag, tag_sha: sha};
}

async function get_release_body({execSync, fs, tag_sha, sha}) {
  // get necessary text
  execSync('git fetch --prune --unshallow');
  const commit_header = execSync(`git log ${tag_sha}..`).toString().trim();
  const changelog = fs.readFileSync(CHANGELOG_FILE, 'utf8');
  const commit_diff =
      execSync(`git diff --word-diff=porcelain ${tag_sha} ${sha}`).toString();

  // link the text
  let body_raw = '## *Commits*:\n\n';
  body_raw += trim_commit_header(commit_header);
  body_raw += '\n\n---\n\n' + changelog;
  body_raw += '\n\n---\n\n## *Git Diff*:\n\n';
  body_raw += `<details><summary>Changes are listed as follows:</summary>\n`;
  body_raw += trim_diff(commit_diff);
  body_raw += '</details>\n';

  if (body_raw.length > MAX_BODY_LENGTH) {
    const body =
        body_raw.substring(0, MAX_BODY_LENGTH - 20) + '\n\n(More)... ...';
    return {body, origin_body: body_raw};
  } else {
    return {body: body_raw, origin_body: body_raw};
  }
}


function trim_commit_header(header) {
  /* sample header:
commit 5d9af644ceb59cd20af6b07d43e5019ae4c5a9db
Author: Wu-Yijun <wuyijun21@mails.ucas.ac.cn>
Date:   Sun May 5 21:41:09 2024 -0700
    test
    second line
    3rd line
commit 9cb24a67498d18d9c0122c6fc11f271aa9228aaf
Author: Wu-Yijun <wuyijun21@mails.ucas.ac.cn>
Date:   Sun May 5 21:39:45 2024 -0700
    tst
*/
  /* sample result:
### test

*2024-05-05 21:41:09 (北美太平洋夏令时间)* by
[Wu-Yijun](mailto:wuyijun21@mails.ucas.ac.cn)

second line
3rd line

### tst

*2024-05-05 21:39:45 (北美太平洋夏令时间)* by
[Wu-Yijun](mailto:wuyijun21@mails.ucas.ac.cn)

*/
  const pattern =
      /commit ([0-9a-f]{40})\nAuthor: (.*) <(.*)>\nDate: (.*)\n\n((?:.|\n)*?)(?=\ncommit|$)/g;
  let result = '';
  let count = 0;
  for (const match of header.matchAll(pattern)) {
    if (count++ == 3) {
      result += '<details><summary>Expand all commits ... </summary>\n\n';
    }
    const [_, sha, author, email, date, message] = match;
    const content = message.split('\n').map(line => line.trim());
    const header = `### ${content[0]}\n\n`;
    const body = content.slice(1).join('\n');
    const trim_date =
        new Date(date).toLocaleString(LOCAL[0], LOCAL[1]) + LOCAL[2];
    result +=
        `${header}*${trim_date}* by [${author}](mailto:${email})\n\n${body}\n\n`;
  }
  if (count > 3) {
    result += '</details>';
  }
  return result;
}

function trim_diff(diff) {
  // return diff;
  const lines = diff.replaceAll('\r', '').split('\n');
  let typed_lines = [];
  let state = 'none';
  //   return diff;
  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];
    if (line.startsWith('diff --git')) {
      // 如果state为none, 则表示当前是第一个diff, 不需要输出 ``` \n\n
      // 如果state不为none, 则表示当前是一个diff的结束, 需要输出 ``` \n\n
      // 我们希望格式为: 输入: diff --git a/xxx b/xxx
      // 输出: ### xxx \n ```bash \n diff --git a/xxx b/xxx \n ``` \n \n
      // ```diff 进入diff状态
      let result = '';
      if (state !== 'none') {
        result += '\n```\n\n';
      }
      result += `\n### ${line.split(' ')[2].slice(2)}\n\n` +
          '```bash\n' + line + '\n```\n\n' +
          '```diff';
      state = 'diff';
      typed_lines.push(['basic', result]);
      continue;
    }
    if (state === 'diff' &&
        (line.startsWith('index ') || line.startsWith('--- ') ||
         line.startsWith('+++ '))) {
      // 如果当前状态为diff, 且当前行以index, ---, +++开头,
      // 则表示这是diff开头, 直接输出加换行
      typed_lines.push(['basic', line]);
      continue;
    }
    if (line.startsWith('@@ ')) {
      // 如果当前行以@@开头, 则表示这是一个定位行, 直接输出加换行
      // 进入正文状态
      state = 'content';
      typed_lines.push(['basic', line]);
      continue;
    }
    // 正文一共4种状态, +, -, 空格, ~开头
    if (line.startsWith(' ')) {
      // 如果当前行以空格开头, 需要判断它的状态,
      // 如果下一行是+或-, 则表示这是一个前缀行, 不加 *
      // 如果上一行是变化行, 则表示这是一个后缀行, 不加 *
      // 如果是下一行以 ~ 开头, 则表示这是正常行, 在行首加上 * , 表示正常行
      if (lines[i + 1].startsWith('+') || lines[i + 1].startsWith('-')) {
        // 如果如果前缀行全都是空格, 直接略去
        if (line.trim() === '') {
          continue;
        }
        typed_lines.push(['prefix', line]);
        continue;
      }
      if (typed_lines[typed_lines.length - 1][0] === 'change') {
        // 如果后缀行全都是空格, 直接略去
        if (line.trim() === '') {
          continue;
        }
        typed_lines.push(['suffix', line]);
        continue;
      }
      typed_lines.push(['basic', '*' + line]);
      continue;
    }
    if (line.startsWith('-') || line.startsWith('+')) {
      // 如果当前行以+-开头, 则表示这是一个发生变化的行, 我们保存+-不变,
      // 在行首额外加上一个换行, 不加 * , 正常在行尾换行
      // 正常在行尾换行, 进入变化行状态
      // 同时, 我们用一个空格将+-和后面的内容分开
      typed_lines.push(['change', `${line[0]} ${line.slice(1)}`]);
      continue;
    }
    if (line.startsWith('~')) {
      // 如果当前行以~开头, 则表示这是换行, 由于我们已经在上面处理了换行,
      // 我们将
      typed_lines[typed_lines.length - 1][3] = true;
      continue;
    }
    if (line === '') {
      typed_lines.push(['basic', '']);
    }
    // 如果当前行不符合以上任何一种情况, 则表示这是一个异常行,
    // 在前面加一个感叹号直接输出
    typed_lines.push(['basic', '! ' + line]);
  }
  // 最后, 我们需要将最后一个diff的后缀加上 ``` \n\n
  if (state !== 'none') {
    typed_lines.push(['basic', '\n```\n\n']);
  }
  // 将处理后的行拼接起来, 其中 basic 组的前后需要额外加上换行
  // 在 non-basic 组内,
  // 如果下一个是 newline, 则需要将最后添加 \t\\n\n
  // 如果下一个是 non-basic, 则需将最后添加 \t\\\\\n
  let result = '';
  let is_state_basic = true;
  var suffixs;
  var wrap_lines;
  const isbasic = (line) => !!line && line[0] === 'basic';
  const isnewline = (line) => !!line && !!line[3];
  const isnonbasic = (line) => !!line &&
      (line[0] === 'change' || line[0] === 'prefix' || line[0] === 'suffix');

  for (let i = 0; i < typed_lines.length; i++) {
    let line = typed_lines[i];
    if (isbasic(line)) {
      // basic
      if (!is_state_basic) {
        result += linkLinesWithSuffix(wrapLines(wrap_lines), suffixs);
        result += '\n';
        is_state_basic = true;
      }
      result += line[1] + '\n';
    } else if (isnonbasic(line)) {
      // non-baisc
      if (is_state_basic) {
        result += '\n';
        is_state_basic = false;
        wrap_lines = [];
        suffixs = [];
      }
      if (isnewline(typed_lines[i])) {
        // result += line[1] + '\t\\n\n';
        wrap_lines.push(line[1]);
        suffixs.push('\t\\n\n');
      } else if (isnonbasic(typed_lines[i + 1])) {
        // result += line[1] + '\t\\\\\n';
        wrap_lines.push(line[1]);
        suffixs.push('\t\\\n');
      } else if (isbasic(typed_lines[i + 1])) {
        // result += line[1] + '\n';
        wrap_lines.push(line[1]);
        suffixs.push('\n');
      }
    }
  }
  console.error(typed_lines);
  return result;
}

function getWidthOfText(text) {
  var fullWidthChars = /[^\x00-\xff]/g;  // 匹配全角字符
  var halfWidthChars = /[\x00-\xff]/g;   // 匹配半角字符
  var fullWidthWidth = 1.5;              // 全角字符宽度
  var halfWidthWidth = 1;                // 半角字符宽度
  var width = 0;

  // 计算全角字符宽度
  var fullWidthMatches = text.match(fullWidthChars);
  if (fullWidthMatches) {
    width += fullWidthMatches.length * fullWidthWidth;
  }

  // 计算半角字符宽度
  var halfWidthMatches = text.match(halfWidthChars);
  if (halfWidthMatches) {
    width += halfWidthMatches.length * halfWidthWidth;
  }

  return width;
}

function wrapLines(lines) {
  const width_indent = 8;
  let lens = [];
  for (let i = 0; i < lines.length; i++) {
    lens.push(getWidthOfText(lines[i]));
  }
  // find the longest line
  let max_len = 0;
  for (let i = 0; i < lens.length; i++) {
    max_len = Math.max(max_len, lens[i]);
  }
  // set max_len to n * width_indent
  max_len = (Math.floor(max_len / width_indent) + 1) * width_indent;
  // wrap the lines
  for (let i = 0; i < lines.length; i++) {
    lines[i] += ' '.repeat(max_len - lens[i]);
  }
  return lines;
}

function linkLinesWithSuffix(lines, suffixs) {
  let res = '';
  for (let i = 0; i < lines.length; i++) {
    res += lines[i] + suffixs[i];
  }
  return res;
}