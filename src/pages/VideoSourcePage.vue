<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Quản Lý Video Nguồn</h2>
    </div>
    <div class="page-body">
      <div class="demo-section">
        <h3>Demo: File System Operations</h3>
        <p class="placeholder-description">Test các tính năng truy cập hệ thống file:</p>
        
        <div class="demo-controls">
          <button class="demo-btn" @click="testReadDirectory">
            📁 Đọc Thư Mục Home
          </button>
          <button class="demo-btn" @click="testGetDownloadDir">
            📥 Lấy Download Directory
          </button>
          <button class="demo-btn" @click="testCheckFileExists">
            ✅ Kiểm Tra File Tồn Tại
          </button>
        </div>
        
        <div v-if="demoResult" class="demo-result" v-html="demoResult"></div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useTauri } from '../composables/useTauri'

const { callCommand } = useTauri()
const demoResult = ref('')

const formatBytes = (bytes) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const testReadDirectory = async () => {
  demoResult.value = '<p>Đang đọc thư mục...</p>'
  
  try {
    const homeDir = await callCommand('get_home_dir')
    const files = await callCommand('read_directory', { path: homeDir })
    
    let html = `<h4>📁 Nội dung thư mục: ${homeDir}</h4>`
    html += "<ul style='list-style: none; padding: 0; margin-top: 10px;'>"
    
    files.slice(0, 10).forEach(file => {
      const icon = file.is_dir ? '📁' : '📄'
      const size = file.size ? ` (${formatBytes(file.size)})` : ''
      html += `<li style='padding: 5px;'>${icon} ${file.name}${size}</li>`
    })
    
    if (files.length > 10) {
      html += `<li style='padding: 5px; color: #64748b;'>... và ${files.length - 10} mục khác</li>`
    }
    
    html += '</ul>'
    demoResult.value = html
  } catch (error) {
    demoResult.value = `<p style='color: red;'>❌ Lỗi: ${error}</p>`
  }
}

const testGetDownloadDir = async () => {
  demoResult.value = '<p>Đang lấy đường dẫn...</p>'
  
  try {
    const downloadDir = await callCommand('get_download_dir')
    demoResult.value = `<p style='color: green;'>✅ Download Directory: <code>${downloadDir}</code></p>`
  } catch (error) {
    demoResult.value = `<p style='color: red;'>❌ Lỗi: ${error}</p>`
  }
}

const testCheckFileExists = async () => {
  demoResult.value = '<p>Đang kiểm tra...</p>'
  
  try {
    const homeDir = await callCommand('get_home_dir')
    const testPath = homeDir + '/.zshrc'
    
    const exists = await callCommand('check_file_exists', { path: testPath })
    const size = exists ? await callCommand('get_file_size', { path: testPath }) : 0
    
    if (exists) {
      demoResult.value = `
        <p style='color: green;'>✅ File tồn tại: <code>${testPath}</code></p>
        <p>📊 Kích thước: ${formatBytes(size)}</p>
      `
    } else {
      demoResult.value = `<p style='color: orange;'>⚠️ File không tồn tại: <code>${testPath}</code></p>`
    }
  } catch (error) {
    demoResult.value = `<p style='color: red;'>❌ Lỗi: ${error}</p>`
  }
}
</script>

