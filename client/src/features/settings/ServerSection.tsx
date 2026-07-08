// 服务器模式配置 — 服务地址 + 连接状态

import { useEffect, useState } from 'react'
import { Info } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Tooltip } from '@/components/ui/tooltip'
import {
  getBackendBaseUrl,
  getDefaultBackendBaseUrl,
  resetBackendBaseUrl,
  setBackendBaseUrl as persistBackendBaseUrl,
} from '@/services/runtimeConfig'
import { reconnectProvider } from '@/services/recorder'
import { getSetting, setSetting } from '@/services/store'

export default function ServerSection() {
  const [backendBaseUrl, setBackendBaseUrl] = useState('')
  const [defaultBackendBaseUrl, setDefaultBackendBaseUrl] = useState('')
  const [serviceMessage, setServiceMessage] = useState('')
  const [serviceTesting, setServiceTesting] = useState(false)
  const [serviceSaving, setServiceSaving] = useState(false)
  const [asrLanguage, setAsrLanguage] = useState('auto')

  useEffect(() => {
    setBackendBaseUrl(getBackendBaseUrl())
    setDefaultBackendBaseUrl(getDefaultBackendBaseUrl())
    void getSetting('server.language', 'auto').then((v) => setAsrLanguage(String(v || 'auto')))
  }, [])

  const normalize = (v: string) => v.trim().replace(/\/+$/, '')

  const handleTest = async () => {
    const normalized = normalize(backendBaseUrl)
    if (!normalized) { setServiceMessage('请先输入服务地址'); return }
    try {
      setServiceTesting(true)
      const response = await fetch(`${normalized}/healthz`, { cache: 'no-store' })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const payload = await response.json() as { status?: string; asr?: boolean; llm?: boolean }
      setServiceMessage(`连接成功：ASR=${payload.asr ? 'on' : 'off'}，LLM=${payload.llm ? 'on' : 'off'}`)
    } catch (error) {
      setServiceMessage(`连接失败：${String(error)}`)
    } finally {
      setServiceTesting(false)
    }
  }

  const handleSaveAndTest = async () => {
    const normalized = normalize(backendBaseUrl)
    if (!normalized) { setServiceMessage('服务地址不能为空'); return }
    try {
      new URL(normalized)
    } catch {
      setServiceMessage('服务地址格式不正确'); return
    }

    // 先保存
    try {
      setServiceSaving(true)
      const next = await persistBackendBaseUrl(normalized)
      setBackendBaseUrl(next)
      // 地址已变更：无论下方健康检查成功与否，都按新地址强制重连，
      // 让左下角连接状态反映新配置（改成错误地址后应显示未连接，而非仍旧"已连接"）
      reconnectProvider()
    } catch (error) {
      setServiceMessage(`保存失败：${String(error)}`); setServiceSaving(false); return
    }

    // 再测试（仅用于给出提示文案，实际连接状态由上面的重连驱动）
    try {
      const response = await fetch(`${normalized}/healthz`, { cache: 'no-store' })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const payload = await response.json() as { status?: string; asr?: boolean; llm?: boolean }
      setServiceMessage(`已保存，连接成功：ASR=${payload.asr ? 'on' : 'off'}，LLM=${payload.llm ? 'on' : 'off'}`)
    } catch (error) {
      setServiceMessage(`已保存，但连接失败：${String(error)}`)
    } finally {
      setServiceSaving(false)
    }
  }

  return (
    <>
    <Card>
      <CardContent className="p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold">服务地址</h2>
            <p className="mt-1 flex items-center gap-1 text-sm text-muted-foreground">
              输入你部署的 SayIt 服务器地址，保存后客户端会自动连接。
              <Tooltip
                variant="light"
                content={'默认地址 sayitapp.site 是作者提供的免费体验服务器，已内置语音识别与 AI 功能，方便快速试用。\n由于服务器运行存在成本，服务不保证长期稳定可用。\n如需稳定使用，建议使用本地识别、接入豆包 API，或自行部署后端服务。'}
              >
                <Info className="h-3.5 w-3.5 shrink-0 cursor-help text-muted-foreground/70 transition-colors hover:text-foreground" />
              </Tooltip>
            </p>
          </div>
        </div>

        <div className="mt-4 space-y-3">
          <div className="flex gap-2">
            <input
              value={backendBaseUrl}
              onChange={(e) => setBackendBaseUrl(e.target.value)}
              placeholder="https://sayitapp.site"
              className="h-9 flex-1 rounded-md border border-input-border bg-input-bg px-3 text-sm outline-none transition-colors focus:border-input-focus-border"
            />
            <Button variant="outline" size="sm" className="h-9 shrink-0" onClick={() => void handleTest()} disabled={serviceTesting}>
              {serviceTesting ? '测试中...' : '测试连接'}
            </Button>
            <Button variant="outline" size="sm" className="h-9 shrink-0" onClick={() => void handleSaveAndTest()} disabled={serviceSaving}>
              {serviceSaving ? '保存中...' : '保存'}
            </Button>
          </div>
        </div>

        {serviceMessage && <p className="mt-2 text-sm text-muted-foreground">{serviceMessage}</p>}
      </CardContent>
    </Card>

    <Card>
      <CardContent className="p-6">
        <h2 className="mb-3 text-lg font-semibold">识别语言</h2>
        <div className="flex gap-2">
          {([
            { value: 'auto', label: '自动' },
            { value: 'zh', label: '中文' },
            { value: 'en', label: '英文' },
          ] as const).map((lang) => (
            <button
              key={lang.value}
              type="button"
              onClick={() => { setAsrLanguage(lang.value); void setSetting('server.language', lang.value) }}
              className={`rounded-md border px-4 py-1.5 text-sm transition-colors ${
                asrLanguage === lang.value
                  ? 'border-primary bg-primary text-primary-foreground'
                  : 'border-border bg-card text-foreground hover:bg-accent'
              }`}
            >
              {lang.label}
            </button>
          ))}
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          大部分场景选"自动"即可。纯英文会议建议选"英文"以提高准确率。
        </p>
      </CardContent>
    </Card>
  </>
  )
}
