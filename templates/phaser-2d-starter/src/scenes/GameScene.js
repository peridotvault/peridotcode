/**
 * GameScene - Main gameplay scene
 * Contains the player, game world, and core gameplay logic
 */

import Player from '../entities/Player.js';

export default class GameScene extends Phaser.Scene {
    constructor() {
        super({ key: 'GameScene' });
    }

    create() {
        console.log('GameScene: Starting gameplay');
        
        // Set world bounds
        this.physics.world.setBounds(0, 0, 1600, 1200);
        
        // Create a simple level with ground tiles
        this.createLevel();
        
        // Create the player
        this.player = new Player(this, 400, 300);
        
        // Set camera to follow player
        this.cameras.main.setBounds(0, 0, 1600, 1200);
        this.cameras.main.startFollow(this.player.sprite, true, 0.1, 0.1);
        this.cameras.main.setZoom(1);
        
        // Add some collectible items
        this.createCollectibles();
        
        // Setup keyboard input
        this.cursors = this.input.keyboard.createCursorKeys();
        
        // Display instructions
        this.add.text(10, 10, '{{game_title}}', {
            fontSize: '24px',
            fill: '#ffffff',
            backgroundColor: '#000000aa',
            padding: { x: 10, y: 5 }
        }).setScrollFactor(0);
        
        this.add.text(10, 50, 'Arrow keys to move', {
            fontSize: '14px',
            fill: '#cccccc',
            backgroundColor: '#000000aa',
            padding: { x: 5, y: 2 }
        }).setScrollFactor(0);
        
        // Score tracking
        this.score = 0;
        this.scoreText = this.add.text(10, 80, 'Score: 0', {
            fontSize: '18px',
            fill: '#ffffff',
            backgroundColor: '#000000aa',
            padding: { x: 5, y: 2 }
        }).setScrollFactor(0);
    }

    update() {
        // Update player with current input state
        this.player.update(this.cursors);
    }
    
    createLevel() {
        // Create static group for ground/platforms
        this.platforms = this.physics.add.staticGroup();
        
        // Create a simple floor
        for (let x = 0; x < 1600; x += 64) {
            for (let y = 600; y < 1200; y += 64) {
                this.platforms.create(x + 32, y + 32, 'ground');
            }
        }
        
        // Add some raised platforms
        const platformPositions = [
            { x: 300, y: 450 },
            { x: 500, y: 350 },
            { x: 700, y: 450 },
            { x: 900, y: 300 },
            { x: 1100, y: 400 },
            { x: 1300, y: 350 }
        ];
        
        platformPositions.forEach(pos => {
            this.platforms.create(pos.x, pos.y, 'ground');
            this.platforms.create(pos.x + 64, pos.y, 'ground');
        });
    }
    
    createCollectibles() {
        // Create a group for collectible items
        this.collectibles = this.physics.add.group({
            key: 'coin',
            repeat: 11,
            setXY: { x: 200, y: 0, stepX: 120 }
        });
        
        // Add bounce animation to collectibles
        this.collectibles.children.iterate((child) => {
            child.setBounceY(Phaser.Math.FloatBetween(0.4, 0.8));
        });
        
        // Add collision between collectibles and platforms
        this.physics.add.collider(this.collectibles, this.platforms);
        
        // Add overlap detection for player collecting items
        this.physics.add.overlap(
            this.player.sprite,
            this.collectibles,
            this.collectItem,
            null,
            this
        );
    }
    
    collectItem(player, item) {
        // Remove the item
        item.disableBody(true, true);
        
        // Update score
        this.score += 10;
        this.scoreText.setText('Score: ' + this.score);
        
        // Simple visual feedback
        this.tweens.add({
            targets: this.scoreText,
            scale: 1.2,
            duration: 100,
            yoyo: true
        });
        
        // Check if all items collected
        if (this.collectibles.countActive(true) === 0) {
            // Respawn all items
            this.collectibles.children.iterate((child) => {
                child.enableBody(true, child.x, 0, true, true);
            });
        }
    }
}
