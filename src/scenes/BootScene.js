export class BootScene extends Phaser.Scene {
    constructor() {
        super({ key: 'BootScene' });
    }

    preload() {
        // Create simple colored textures programmatically
        const graphics = this.make.graphics();
        
        // Player texture (red square)
        graphics.fillStyle(0xff0000);
        graphics.fillRect(0, 0, 32, 32);
        graphics.generateTexture('player', 32, 32);
        
        // Platform texture (green)
        graphics.clear();
        graphics.fillStyle(0x00ff00);
        graphics.fillRect(0, 0, 400, 32);
        graphics.generateTexture('platform', 400, 32);
        
        // Ground texture
        graphics.clear();
        graphics.fillStyle(0x654321);
        graphics.fillRect(0, 0, 800, 64);
        graphics.generateTexture('ground', 800, 64);
        
        graphics.destroy();
        
        console.log('Assets loaded');
    }

    create() {
        this.scene.start('GameScene');
    }
}