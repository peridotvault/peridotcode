import { Player } from '../entities/Player.js';

export class GameScene extends Phaser.Scene {
    constructor() {
        super({ key: 'GameScene' });
    }

    create() {
        // Background
        this.add.rectangle(400, 300, 800, 600, 0x87CEEB);
        
        // Create platforms
        const platforms = this.physics.add.staticGroup();
        
        // Ground
        platforms.create(400, 568, 'ground').setScale(1).refreshBody();
        
        // Ledges
        platforms.create(600, 400, 'platform').setScale(0.5).refreshBody();
        platforms.create(50, 250, 'platform').setScale(0.5).refreshBody();
        platforms.create(750, 220, 'platform').setScale(0.5).refreshBody();
        
        // Create player
        this.player = new Player(this, 100, 450);
        
        // Collisions
        this.physics.add.collider(this.player.sprite, platforms);
        
        // Instructions
        this.add.text(400, 50, 'peridotcode', {
            fontSize: '32px',
            fill: '#000'
        }).setOrigin(0.5);
        
        this.add.text(400, 100, 'Arrow keys to move, Space to jump', {
            fontSize: '16px',
            fill: '#000'
        }).setOrigin(0.5);
    }

    update() {
        this.player.update();
    }
}