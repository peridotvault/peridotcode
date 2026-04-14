/**
 * BootScene - Asset loading and initialization
 * This scene loads before the main game and sets up any assets
 */

export default class BootScene extends Phaser.Scene {
    constructor() {
        super({ key: 'BootScene' });
    }

    preload() {
        // Display loading text
        const width = this.cameras.main.width;
        const height = this.cameras.main.height;
        
        const loadingText = this.add.text(width / 2, height / 2, 'Loading...', {
            fontSize: '32px',
            fill: '#ffffff'
        }).setOrigin(0.5);

        // Create simple placeholder graphics for the player
        // In a real game, you would load actual image files here
        const graphics = this.make.graphics({ x: 0, y: 0, add: false });
        
        // Player sprite (32x32 blue square)
        graphics.fillStyle(0x4a90d9, 1);
        graphics.fillRect(0, 0, 32, 32);
        graphics.generateTexture('player', 32, 32);
        
        // Ground tile (64x64 gray square)
        graphics.clear();
        graphics.fillStyle(0x3d3d5c, 1);
        graphics.fillRect(0, 0, 64, 64);
        graphics.lineStyle(2, 0x4a4a6a, 1);
        graphics.strokeRect(0, 0, 64, 64);
        graphics.generateTexture('ground', 64, 64);
        
        // Collectible item (16x16 yellow circle)
        graphics.clear();
        graphics.fillStyle(0xf0c040, 1);
        graphics.fillCircle(8, 8, 8);
        graphics.generateTexture('coin', 16, 16);
    }

    create() {
        console.log('BootScene: Assets loaded');
        
        // Transition to the main game scene
        this.scene.start('GameScene');
    }
}
