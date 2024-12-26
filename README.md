
[![](https://img.shields.io/crates/v/examination)](https://crates.io/crates/examination)
[![](https://img.shields.io/crates/l/examination)](https://www.apache.org/licenses/LICENSE-2.0.txt)
[![](https://img.shields.io/github/workflow/status/quyc07/examination/examination/ci)](https://github.com/quyc07/examination/actions/workflows/ci.yml)

# examination

一个简单的考试命令行工具，通过读取本地试题库json文件生成考试题目，支持用户通过命令行进行作答，提交考卷后可计算得分，并高亮显示错题。

## 特性

1. 支持本地试题库文件读取，支持自定义试题库文件路径
2. 支持单选题，多选题，填空题，判断题
3. 支持自动计算分数
4. 支持错误题目对比
5. 支持自定义题目数量
6. 支持随机出题
7. 支持自定义考试名称

## 安装

可使用cargo直接安装

```shell
cargo install examination
```

也可直接下载对应平台的可执行文件

## 运行

```shell
examination
```

## 配置

- 默认数据路径为 `.data`，可以通过环境变量 `EXAMINATION_PATH` 来修改
- 默认试题库文件路径为 `.data/question.json`
- 默认考试配置文件为 `.data/examination.json`
- 默认配置路径为 `.config`，可以通过环境变量 `EXAMINATION_CONFIG` 来修改
- 默认快捷键配置文件为 `.config/config.json5`

## 试题库格式

参考 [question.json](./.data/question.json)

## 考试配置格式

参考 [examination.json](./.data/examination.json)

## Roadmap

- [X] 支持多选，填空题，判断题
- [X] 支持考试计时
- [X] 支持从题库中随机出题
- [ ] 支持错题记录
- [ ] 支持错题重做




