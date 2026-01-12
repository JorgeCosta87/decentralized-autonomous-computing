import yaml from 'js-yaml';

export interface IpfsClientOptions {
  apiUrl?: string;
}

export class IpfsClient {
  private apiUrl: string;

  constructor(options: IpfsClientOptions = {}) {
    this.apiUrl = options.apiUrl || 'http://localhost:5001';
  }

  /**
   * Upload data to IPFS and pin it
   * Optionally adds to MFS (Mutable File System) so it appears in WebUI
   */
  async upload(data: string | object, filename?: string, addToMfs: boolean = true): Promise<string> {
    const content = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
    
    const formData = new FormData();
    const blob = new Blob([content], { type: 'application/json' });
    const finalFilename = filename || 'data.json';
    formData.append('file', blob, finalFilename);

    const response = await fetch(`${this.apiUrl}/api/v0/add?pin=true&wrap-with-directory=false`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`IPFS upload failed: ${error}`);
    }

    const result = await response.json();
    const cid = result.Hash;

    // Add to MFS so it appears in WebUI Files view
    if (addToMfs) {
      try {
        await this.addToMfs(cid, finalFilename);
      } catch (error) {
        console.warn(`Failed to add ${cid} to MFS (file still pinned):`, error);
      }
    }

    return cid;
  }

  /**
   * Add a CID to IPFS MFS (Mutable File System) so it appears in WebUI
   */
  async addToMfs(cid: string, filename: string): Promise<void> {
    // Create directory structure: /dac-uploads/YYYY-MM-DD/filename
    const date = new Date().toISOString().split('T')[0]; // YYYY-MM-DD
    const mfsPath = `/dac-uploads/${date}/${filename}`;

    // Ensure parent directory exists
    try {
      await fetch(`${this.apiUrl}/api/v0/files/mkdir?arg=/dac-uploads&parents=true`, {
        method: 'POST',
      });
    } catch {
      // Directory might already exist, ignore
    }

    try {
      await fetch(`${this.apiUrl}/api/v0/files/mkdir?arg=/dac-uploads/${date}&parents=true`, {
        method: 'POST',
      });
    } catch {
      // Directory might already exist, ignore
    }

    // Remove existing file if it exists (to avoid "directory already has entry by that name" error)
    try {
      const statResponse = await fetch(`${this.apiUrl}/api/v0/files/stat?arg=${mfsPath}`, {
        method: 'POST',
      });
      if (statResponse.ok) {
        // File exists, remove it first
        await fetch(`${this.apiUrl}/api/v0/files/rm?arg=${mfsPath}`, {
          method: 'POST',
        });
      }
    } catch {
      // File doesn't exist, which is fine - we'll create it
    }

    // Copy CID to MFS path
    const response = await fetch(
      `${this.apiUrl}/api/v0/files/cp?arg=/ipfs/${cid}&arg=${mfsPath}`,
      { method: 'POST' }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to add to MFS: ${error}`);
    }
  }

  /**
   * Download data from IPFS
   */
  async download(cid: string): Promise<string> {
    const response = await fetch(`${this.apiUrl}/api/v0/cat?arg=${cid}`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`IPFS download failed: ${error}`);
    }

    return await response.text();
  }

  /**
   * Pin a CID
   */
  async pin(cid: string): Promise<void> {
    const response = await fetch(`${this.apiUrl}/api/v0/pin/add?arg=${cid}&recursive=true`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`IPFS pin failed: ${error}`);
    }
  }

  /**
   * Load and upload YAML config file
   */
  async uploadYamlConfig(yamlContent: string, filename?: string): Promise<string> {
    const config = yaml.load(yamlContent) as object;
    return this.upload(config, filename || 'config.json');
  }

  /**
   * List all pinned CIDs
   */
  async listPinned(): Promise<string[]> {
    const response = await fetch(`${this.apiUrl}/api/v0/pin/ls?type=recursive`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`IPFS list pinned failed: ${error}`);
    }

    const result = await response.json();
    return Object.keys(result.Keys || {});
  }

  /**
   * Get file info from IPFS
   */
  async stat(cid: string): Promise<any> {
    const response = await fetch(`${this.apiUrl}/api/v0/files/stat?arg=${cid}`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`IPFS stat failed: ${error}`);
    }

    return await response.json();
  }

  /**
   * List all files in MFS directory (for easy tracking)
   */
  async listMfsFiles(path: string = '/dac-uploads'): Promise<any[]> {
    const response = await fetch(`${this.apiUrl}/api/v0/files/ls?arg=${path}`, {
      method: 'POST',
    });

    if (!response.ok) {
      // Directory might not exist yet
      if (response.status === 404) {
        return [];
      }
      const error = await response.text();
      throw new Error(`IPFS MFS list failed: ${error}`);
    }

    const result = await response.json();
    return result.Entries || [];
  }

  /**
   * Get detailed info about all pinned files with metadata
   */
  async listPinnedWithDetails(): Promise<Array<{
    cid: string;
    gatewayUrl: string;
    apiUrl: string;
    size?: number;
    type?: string;
  }>> {
    const cids = await this.listPinned();
    
    return cids.map(cid => ({
      cid,
      gatewayUrl: `http://localhost:8080/ipfs/${cid}`,
      apiUrl: `http://localhost:5001/api/v0/cat?arg=${cid}`,
    }));
  }

  /**
   * Get gateway URL for a CID
   */
  getGatewayUrl(cid: string, gatewayPort: number = 8080): string {
    return `http://localhost:${gatewayPort}/ipfs/${cid}`;
  }

  /**
   * Get API URL for a CID
   */
  getApiUrl(cid: string): string {
    return `${this.apiUrl}/api/v0/cat?arg=${cid}`;
  }
}
